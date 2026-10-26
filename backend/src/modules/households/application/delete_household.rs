use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::{HouseholdApplicationEvent, HouseholdId},
            ports::{HouseholdAccessError, HouseholdAccessPolicy, HouseholdRepository},
        },
    },
    shared::application::{InternalError, events::ApplicationResult},
};

pub struct DeleteHouseholdCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
}

pub struct DeleteHouseholdService {
    household_repository: Arc<dyn HouseholdRepository>,
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
}

impl DeleteHouseholdService {
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
        command: DeleteHouseholdCommand,
    ) -> Result<ApplicationResult<()>, DeleteHouseholdError> {
        self.household_access_policy
            .require_owner(&command.household_id, &command.requester_id)
            .await?;

        let members = self
            .household_repository
            .find_members(&command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to retrieve members for household"
                );
                DeleteHouseholdError::Internal(InternalError::Failed)
            })?;

        if members.len() > 1 {
            return Err(DeleteHouseholdError::HouseholdHasOtherMembers);
        }

        self.household_repository
            .delete(&command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    requester_id = %command.requester_id,
                    "Failed to delete household",
                );
                DeleteHouseholdError::Internal(InternalError::Failed)
            })?;

        Ok(
            ApplicationResult::new(()).with_event(HouseholdApplicationEvent::Deleted {
                household_id: command.household_id,
                owner_id: command.requester_id,
            }),
        )
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DeleteHouseholdError {
    #[error("The household cannot be deleted while it has other members")]
    HouseholdHasOtherMembers,
    #[error(transparent)]
    Internal(#[from] InternalError),
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
}

#[cfg(test)]
mod tests {
    use crate::{
        modules::households::{
            adapters::{DefaultHouseholdAccessPolicy, InMemoryHouseholdRepository},
            domain::{HouseholdApplicationEvent, HouseholdKind},
            ports::HouseholdAccessError,
        },
        shared::application::{InternalError, events::AppEvent},
        test_helpers::{
            FailingHouseholdRepository, build_delete_household_service, create_owned_household,
            insert_member, insert_owned_household,
        },
    };

    use super::*;

    #[tokio::test]
    async fn owner_can_delete_household() {
        let (service, household_repository) = build_delete_household_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        service
            .execute(DeleteHouseholdCommand {
                requester_id: owner_id,
                household_id: household.id(),
            })
            .await
            .expect("Household deletion should succeed");

        let stored = household_repository
            .find_by_id(&household.id())
            .await
            .expect("Household lookup should succeed");

        assert!(stored.is_none());
    }

    #[tokio::test]
    async fn deleting_household_emits_deleted_event() {
        let (service, household_repository) = build_delete_household_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let result = service
            .execute(DeleteHouseholdCommand {
                requester_id: owner_id,
                household_id: household.id(),
            })
            .await
            .expect("Household deletion should succeed");

        let (_, events) = result.into_parts();

        assert_eq!(
            events,
            vec![AppEvent::Household(HouseholdApplicationEvent::Deleted {
                household_id: household.id(),
                owner_id
            })]
        );
    }

    #[tokio::test]
    async fn household_with_other_members_cannot_be_deleted() {
        let (service, household_repository) = build_delete_household_service();

        let owner_id = UserId::new();
        let member_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        insert_member(&household_repository, household.id(), member_id).await;

        let result = service
            .execute(DeleteHouseholdCommand {
                requester_id: owner_id,
                household_id: household.id(),
            })
            .await;

        assert!(matches!(
            result,
            Err(DeleteHouseholdError::HouseholdHasOtherMembers)
        ));

        let stored = household_repository
            .find_by_id(&household.id())
            .await
            .expect("Household lookup should succeed");

        assert!(stored.is_some());
    }

    #[tokio::test]
    async fn non_owner_is_forbidden() {
        let (service, household_repository) = build_delete_household_service();

        let owner_id = UserId::new();
        let member_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        insert_member(&household_repository, household.id(), member_id).await;

        let result = service
            .execute(DeleteHouseholdCommand {
                requester_id: member_id,
                household_id: household.id(),
            })
            .await;

        assert!(matches!(
            result,
            Err(DeleteHouseholdError::HouseholdAccess(
                HouseholdAccessError::Forbidden
            ))
        ));
    }

    #[tokio::test]
    async fn unknown_household_returns_not_found() {
        let (service, _) = build_delete_household_service();

        let result = service
            .execute(DeleteHouseholdCommand {
                requester_id: UserId::new(),
                household_id: HouseholdId::new(),
            })
            .await;

        assert!(matches!(
            result,
            Err(DeleteHouseholdError::HouseholdAccess(
                HouseholdAccessError::HouseholdNotFound
            ))
        ));
    }

    #[tokio::test]
    async fn repository_failure_returns_internal() {
        let access_repository = Arc::new(InMemoryHouseholdRepository::new());

        let owner_id = UserId::new();

        let (household, owner) = create_owned_household(owner_id, HouseholdKind::Shared);

        access_repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let household_access_policy =
            Arc::new(DefaultHouseholdAccessPolicy::new(access_repository));

        let repository = Arc::new(FailingHouseholdRepository);

        let service = DeleteHouseholdService::new(repository, household_access_policy);

        let result = service
            .execute(DeleteHouseholdCommand {
                requester_id: owner_id,
                household_id: household.id(),
            })
            .await;

        assert!(matches!(
            result,
            Err(DeleteHouseholdError::Internal(InternalError::Failed))
        ));
    }
}
