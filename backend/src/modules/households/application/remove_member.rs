use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::{HouseholdId, HouseholdRole},
            ports::{
                HouseholdAccessError, HouseholdAccessPolicy, HouseholdRepository,
                HouseholdRepositoryError,
            },
        },
    },
    shared::application::InternalError,
};

pub struct RemoveHouseholdMemberCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub member_id: UserId,
}

pub struct RemoveHouseholdMemberService {
    household_repository: Arc<dyn HouseholdRepository>,
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
}

impl RemoveHouseholdMemberService {
    pub fn new(
        household_repository: Arc<dyn HouseholdRepository>,
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    ) -> Self {
        Self {
            household_repository,
            household_access_policy,
        }
    }

    pub async fn execute(
        &self,
        command: RemoveHouseholdMemberCommand,
    ) -> Result<(), RemoveHouseholdMemberError> {
        let requester = self
            .household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        if !(command.requester_id == command.member_id) && requester.role() != HouseholdRole::Owner
        {
            return Err(RemoveHouseholdMemberError::HouseholdAccess(
                HouseholdAccessError::Forbidden,
            ));
        }

        let target = self
            .household_repository
            .find_member(&command.household_id, &command.member_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    member_id = %command.member_id,
                    "Failed to load target membership",
                );
                InternalError::Failed
            })?
            .ok_or(RemoveHouseholdMemberError::MemberNotFound)?;

        if target.role() == HouseholdRole::Owner {
            return Err(RemoveHouseholdMemberError::OwnerCannotBeRemoved);
        }

        self.household_repository
            .remove_member(&command.household_id, &command.member_id)
            .await
            .map_err(|error| match error {
                HouseholdRepositoryError::MemberNotFound => {
                    RemoveHouseholdMemberError::MemberNotFound
                }
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        member_id = %command.member_id,
                        "Failed to remove household member"
                    );
                    RemoveHouseholdMemberError::Internal(InternalError::Failed)
                }
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RemoveHouseholdMemberError {
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error("Household member not found")]
    MemberNotFound,
    #[error("The household owner cannot be removed")]
    OwnerCannotBeRemoved,
    #[error(transparent)]
    Internal(#[from] InternalError),
}

#[cfg(test)]
mod tests {

    use chrono::Utc;

    use super::*;
    use crate::{
        modules::households::{
            adapters::{DefaultHouseholdAccessPolicy, InMemoryHouseholdRepository},
            application::RemoveHouseholdMemberCommand,
            domain::{HouseholdKind, HouseholdMember},
            ports::HouseholdRepository,
        },
        test_helpers::{
            FailingHouseholdRepository, MissingOnRemoveHouseholdRepository,
            build_remove_household_member_service, create_owned_household,
            create_shared_household_fixture, insert_member, insert_user,
        },
    };

    #[tokio::test]
    async fn owner_can_remove_member() {
        let (service, household_repository, user_repository) =
            build_remove_household_member_service();

        let fixture =
            create_shared_household_fixture(&user_repository, &household_repository).await;

        let member = insert_user(&user_repository, "member@email.com").await;

        insert_member(&household_repository, fixture.household.id(), member.id()).await;

        let result = service
            .execute(RemoveHouseholdMemberCommand {
                requester_id: fixture.owner.id(),
                household_id: fixture.household.id(),
                member_id: member.id(),
            })
            .await;

        assert!(result.is_ok());

        let stored = household_repository
            .find_member(&fixture.household.id(), &member.id())
            .await
            .expect("Membership lookup should succeed");

        assert!(stored.is_none())
    }

    #[tokio::test]
    async fn member_can_remove_themselves() {
        let (service, household_repository, user_repository) =
            build_remove_household_member_service();

        let fixture =
            create_shared_household_fixture(&user_repository, &household_repository).await;

        let member = insert_user(&user_repository, "member@email.com").await;

        insert_member(&household_repository, fixture.household.id(), member.id()).await;

        let result = service
            .execute(RemoveHouseholdMemberCommand {
                requester_id: member.id(),
                household_id: fixture.household.id(),
                member_id: member.id(),
            })
            .await;

        assert!(result.is_ok());

        let stored = household_repository
            .find_member(&fixture.household.id(), &member.id())
            .await
            .expect("Membership lookup should succeed");

        assert!(stored.is_none())
    }

    #[tokio::test]
    async fn member_cannot_remove_another_member() {
        let (service, household_repository, user_repository) =
            build_remove_household_member_service();

        let fixture =
            create_shared_household_fixture(&user_repository, &household_repository).await;

        let requester = insert_user(&user_repository, "requester@email.com").await;
        let target = insert_user(&user_repository, "target@email.com").await;

        insert_member(
            &household_repository,
            fixture.household.id(),
            requester.id(),
        )
        .await;
        insert_member(&household_repository, fixture.household.id(), target.id()).await;

        let result = service
            .execute(RemoveHouseholdMemberCommand {
                requester_id: requester.id(),
                household_id: fixture.household.id(),
                member_id: target.id(),
            })
            .await;

        assert_eq!(
            result,
            Err(RemoveHouseholdMemberError::HouseholdAccess(
                HouseholdAccessError::Forbidden
            ))
        );

        let stored = household_repository
            .find_member(&fixture.household.id(), &target.id())
            .await
            .expect("Membership lookup should succeed");

        assert!(stored.is_some())
    }

    #[tokio::test]
    async fn non_member_requester_returns_forbidden() {
        let (service, household_repository, user_repository) =
            build_remove_household_member_service();

        let fixture =
            create_shared_household_fixture(&user_repository, &household_repository).await;

        let requester = insert_user(&user_repository, "requester@email.com").await;
        let target = insert_user(&user_repository, "target@email.com").await;

        insert_member(&household_repository, fixture.household.id(), target.id()).await;

        let result = service
            .execute(RemoveHouseholdMemberCommand {
                requester_id: requester.id(),
                household_id: fixture.household.id(),
                member_id: target.id(),
            })
            .await;

        assert_eq!(
            result,
            Err(RemoveHouseholdMemberError::HouseholdAccess(
                HouseholdAccessError::Forbidden
            ))
        );

        let stored = household_repository
            .find_member(&fixture.household.id(), &target.id())
            .await
            .expect("Membership lookup should succeed");

        assert!(stored.is_some())
    }

    #[tokio::test]
    async fn unknown_household_returns_household_not_found() {
        let (service, _, _) = build_remove_household_member_service();

        let result = service
            .execute(RemoveHouseholdMemberCommand {
                requester_id: UserId::new(),
                household_id: HouseholdId::new(),
                member_id: UserId::new(),
            })
            .await;

        assert_eq!(
            result,
            Err(RemoveHouseholdMemberError::HouseholdAccess(
                HouseholdAccessError::HouseholdNotFound
            ))
        );
    }

    #[tokio::test]
    async fn unknown_target_member_returns_member_not_found() {
        let (service, household_repository, user_repository) =
            build_remove_household_member_service();

        let fixture =
            create_shared_household_fixture(&user_repository, &household_repository).await;

        let target = insert_user(&user_repository, "target@email.com").await;

        let result = service
            .execute(RemoveHouseholdMemberCommand {
                requester_id: fixture.owner.id(),
                household_id: fixture.household.id(),
                member_id: target.id(),
            })
            .await;

        assert_eq!(result, Err(RemoveHouseholdMemberError::MemberNotFound));
    }

    #[tokio::test]
    async fn owner_cannot_remove_themselves() {
        let (service, household_repository, user_repository) =
            build_remove_household_member_service();

        let fixture =
            create_shared_household_fixture(&user_repository, &household_repository).await;

        let result = service
            .execute(RemoveHouseholdMemberCommand {
                requester_id: fixture.owner.id(),
                household_id: fixture.household.id(),
                member_id: fixture.owner.id(),
            })
            .await;

        assert_eq!(
            result,
            Err(RemoveHouseholdMemberError::OwnerCannotBeRemoved)
        );

        let stored = household_repository
            .find_member(&fixture.household.id(), &fixture.owner.id())
            .await
            .expect("Membership lookup should succeed");

        assert!(stored.is_some())
    }

    #[tokio::test]
    async fn household_repository_failure_returns_internal() {
        let repository = Arc::new(FailingHouseholdRepository);
        let policy = Arc::new(DefaultHouseholdAccessPolicy::new(repository.clone()));
        let service = RemoveHouseholdMemberService::new(repository, policy);

        let result = service
            .execute(RemoveHouseholdMemberCommand {
                requester_id: UserId::new(),
                household_id: HouseholdId::new(),
                member_id: UserId::new(),
            })
            .await;

        assert_eq!(
            result,
            Err(RemoveHouseholdMemberError::HouseholdAccess(
                HouseholdAccessError::Internal(InternalError::Failed)
            ))
        )
    }

    #[tokio::test]
    async fn member_not_found_from_repository_is_preserved() {
        let inner = Arc::new(InMemoryHouseholdRepository::new());

        let repository = Arc::new(MissingOnRemoveHouseholdRepository {
            inner: inner.clone(),
        });

        let policy = Arc::new(DefaultHouseholdAccessPolicy::new(repository.clone()));
        let service = RemoveHouseholdMemberService::new(repository, policy);

        let owner_id = UserId::new();
        let member_id = UserId::new();

        let (household, owner) = create_owned_household(owner_id, HouseholdKind::Shared);

        inner
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let member =
            HouseholdMember::new(household.id(), member_id, HouseholdRole::Member, Utc::now());

        inner
            .add_member(&member)
            .await
            .expect("Adding member should succeed");

        let result = service
            .execute(RemoveHouseholdMemberCommand {
                requester_id: owner_id,
                household_id: household.id(),
                member_id,
            })
            .await;

        assert_eq!(result, Err(RemoveHouseholdMemberError::MemberNotFound))
    }
}
