use std::{collections::HashMap, sync::Arc};

use crate::{
    modules::{
        accounts::{
            domain::{DisplayName, UserId},
            ports::UserRepository,
        },
        households::{
            domain::{HouseholdId, HouseholdMember, HouseholdRole},
            ports::{HouseholdAccessError, HouseholdAccessPolicy, HouseholdRepository},
        },
    },
    shared::application::InternalError,
};

pub struct ListHouseholdMembersCommand {
    pub household_id: HouseholdId,
    pub requester_id: UserId,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HouseholdMemberInfo {
    pub user_id: UserId,
    pub display_name: DisplayName,
    pub role: HouseholdRole,
}

pub struct ListHouseholdMembersService {
    household_repository: Arc<dyn HouseholdRepository>,
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    user_repository: Arc<dyn UserRepository>,
}

impl ListHouseholdMembersService {
    pub fn new(
        household_repository: Arc<dyn HouseholdRepository>,
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            household_repository,
            household_access_policy,
            user_repository,
        }
    }

    pub async fn execute(
        &self,
        command: ListHouseholdMembersCommand,
    ) -> Result<Vec<HouseholdMemberInfo>, ListHouseholdMembersError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let members = self
            .household_repository
            .find_members(&command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "failed to load household members",
                );
                InternalError::Failed
            })?;

        let user_ids: Vec<UserId> = members.iter().map(HouseholdMember::user_id).collect();

        let users = self
            .user_repository
            .find_by_ids(&user_ids)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to load users for household members"
                );
                InternalError::Failed
            })?;

        let users_by_id: HashMap<UserId, DisplayName> = users
            .into_iter()
            .map(|user| (user.id(), user.display_name().clone()))
            .collect();

        let member_infos = members
            .into_iter()
            .map(|member| {
                let display_name = users_by_id.get(&member.user_id()).ok_or_else(|| {
                    tracing::error!(
                        household_id = %command.household_id,
                        user_id = %member.user_id(),
                        "Household member references missing user",
                    );
                    ListHouseholdMembersError::Internal(InternalError::Failed)
                })?;

                Ok(HouseholdMemberInfo {
                    user_id: member.user_id(),
                    display_name: display_name.clone(),
                    role: member.role(),
                })
            })
            .collect::<Result<Vec<_>, ListHouseholdMembersError>>()?;

        Ok(member_infos)
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ListHouseholdMembersError {
    #[error("No household was found")]
    NotFound,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

#[cfg(test)]
mod tests {

    use crate::{
        modules::{
            accounts::adapters::InMemoryUserRepository,
            households::{
                adapters::{DefaultHouseholdAccessPolicy, InMemoryHouseholdRepository},
                domain::HouseholdKind,
            },
        },
        test_helpers::{
            FailingHouseholdRepository, FailingUserRepository, MissingUserRepository,
            build_list_household_members_service, create_owned_household, insert_member,
            insert_owned_household, insert_user,
        },
    };

    use super::*;

    #[tokio::test]
    async fn member_can_list_household_member() {
        let (service, household_repository, user_repository) =
            build_list_household_members_service();

        let owner = insert_user(&user_repository, "owner@email.com").await;
        let member = insert_user(&user_repository, "member@email.com").await;

        let (household, _) =
            insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

        insert_member(&household_repository, household.id(), member.id()).await;

        let command = ListHouseholdMembersCommand {
            household_id: household.id(),
            requester_id: owner.id(),
        };

        let result = service
            .execute(command)
            .await
            .expect("Listing members should succeed");

        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn returned_members_include_correct_display_names_and_roles() {
        let (service, household_repository, user_repository) =
            build_list_household_members_service();

        let owner = insert_user(&user_repository, "owner@email.com").await;
        let member = insert_user(&user_repository, "member@email.com").await;

        let (household, _) =
            insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

        insert_member(&household_repository, household.id(), member.id()).await;

        let command = ListHouseholdMembersCommand {
            household_id: household.id(),
            requester_id: owner.id(),
        };

        let result = service
            .execute(command)
            .await
            .expect("Listing members should succeed");

        assert_eq!(result.len(), 2);
        assert!(
            result
                .iter()
                .all(|member| member.display_name.as_str() == "valid name")
        );
        assert!(
            result
                .iter()
                .any(|member| member.role == HouseholdRole::Member)
        );
        assert!(
            result
                .iter()
                .any(|member| member.role == HouseholdRole::Owner)
        );
    }

    #[tokio::test]
    async fn unknown_household_returns_not_found() {
        let (service, _, _) = build_list_household_members_service();

        let command = ListHouseholdMembersCommand {
            household_id: HouseholdId::new(),
            requester_id: UserId::new(),
        };

        let result = service.execute(command).await;

        assert_eq!(
            result,
            Err(ListHouseholdMembersError::HouseholdAccess(
                HouseholdAccessError::HouseholdNotFound
            ))
        );
    }

    #[tokio::test]
    async fn non_member_returns_forbidden() {
        let (service, household_repository, _) = build_list_household_members_service();

        let owner_id = UserId::new();
        let (household, owner_membership) = create_owned_household(owner_id, HouseholdKind::Shared);

        household_repository
            .create_with_owner(&household, &owner_membership)
            .await
            .expect("Household creation should succeed");

        let command = ListHouseholdMembersCommand {
            household_id: household.id(),
            requester_id: UserId::new(),
        };

        let result = service.execute(command).await;

        assert_eq!(
            result,
            Err(ListHouseholdMembersError::HouseholdAccess(
                HouseholdAccessError::Forbidden
            ))
        )
    }

    #[tokio::test]
    async fn household_repository_failure_returns_internal() {
        let household_repository = Arc::new(FailingHouseholdRepository);
        let policy = Arc::new(DefaultHouseholdAccessPolicy::new(
            household_repository.clone(),
        ));
        let user_repository = Arc::new(InMemoryUserRepository::new());

        let service =
            ListHouseholdMembersService::new(household_repository, policy, user_repository);

        let command = ListHouseholdMembersCommand {
            household_id: HouseholdId::new(),
            requester_id: UserId::new(),
        };

        let result = service.execute(command).await;

        assert_eq!(
            result,
            Err(ListHouseholdMembersError::HouseholdAccess(
                HouseholdAccessError::Internal(InternalError::Failed)
            ))
        )
    }

    #[tokio::test]
    async fn user_repository_failure_returns_failure() {
        let household_repository = Arc::new(InMemoryHouseholdRepository::new());
        let policy = Arc::new(DefaultHouseholdAccessPolicy::new(
            household_repository.clone(),
        ));
        let user_repository = Arc::new(FailingUserRepository);

        let service =
            ListHouseholdMembersService::new(household_repository.clone(), policy, user_repository);

        let owner_id = UserId::new();

        let (household, owner_membership) = create_owned_household(owner_id, HouseholdKind::Shared);

        household_repository
            .create_with_owner(&household, &owner_membership)
            .await
            .expect("Household creation should succeed");

        let command = ListHouseholdMembersCommand {
            household_id: household.id(),
            requester_id: owner_id,
        };

        let result = service.execute(command).await;

        assert_eq!(
            result,
            Err(ListHouseholdMembersError::Internal(InternalError::Failed))
        )
    }

    #[tokio::test]
    async fn missing_user_for_an_existing_membership_returns_internal() {
        let household_repository = Arc::new(InMemoryHouseholdRepository::new());
        let policy = Arc::new(DefaultHouseholdAccessPolicy::new(
            household_repository.clone(),
        ));
        let user_repository = Arc::new(MissingUserRepository);

        let service =
            ListHouseholdMembersService::new(household_repository.clone(), policy, user_repository);

        let owner_id = UserId::new();

        let (household, owner_membership) = create_owned_household(owner_id, HouseholdKind::Shared);

        household_repository
            .create_with_owner(&household, &owner_membership)
            .await
            .expect("Household creation should succeed");

        let command = ListHouseholdMembersCommand {
            household_id: household.id(),
            requester_id: owner_id,
        };

        let result = service.execute(command).await;

        assert_eq!(
            result,
            Err(ListHouseholdMembersError::Internal(InternalError::Failed))
        )
    }
}
