use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::{HouseholdApplicationEvent, HouseholdId, HouseholdRole},
            ports::{
                HouseholdAccessError, HouseholdAccessPolicy, HouseholdRepository,
                HouseholdRepositoryError,
            },
        },
    },
    shared::application::{InternalError, events::ApplicationResult},
};

pub struct LeaveHouseholdCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
}

pub struct LeaveHouseholdService {
    household_repository: Arc<dyn HouseholdRepository>,
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
}

impl LeaveHouseholdService {
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
        command: LeaveHouseholdCommand,
    ) -> Result<ApplicationResult<()>, LeaveHouseholdError> {
        let requester = self
            .household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        if requester.role() == HouseholdRole::Owner {
            return Err(LeaveHouseholdError::OwnerCannotLeave);
        }

        self.household_repository
            .remove_member(&command.household_id, &command.requester_id)
            .await
            .map_err(|error| match error {
                HouseholdRepositoryError::MemberNotFound => {
                    LeaveHouseholdError::HouseholdAccess(HouseholdAccessError::Forbidden)
                }
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        member_id = %command.requester_id,
                        "Failed to remove membership",
                    );
                    LeaveHouseholdError::Internal(InternalError::Failed)
                }
            })?;

        Ok(
            ApplicationResult::new(()).with_event(HouseholdApplicationEvent::MemberLeft {
                household_id: command.household_id,
                member_id: command.requester_id,
            }),
        )
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum LeaveHouseholdError {
    #[error("The household owner cannot leave")]
    OwnerCannotLeave,
    #[error(transparent)]
    Internal(#[from] InternalError),
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        modules::households::{
            adapters::{DefaultHouseholdAccessPolicy, InMemoryHouseholdRepository},
            domain::{HouseholdApplicationEvent, HouseholdKind},
        },
        shared::application::events::AppEvent,
        test_helpers::{
            FailingHouseholdRepository, build_leave_household_service, insert_member,
            insert_owned_household,
        },
    };

    use super::*;

    #[tokio::test]
    async fn member_can_leave_shared_household() {
        let (service, household_repository) = build_leave_household_service();

        let owner_id = UserId::new();
        let member_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        insert_member(&household_repository, household.id(), member_id).await;

        let result = service
            .execute(LeaveHouseholdCommand {
                requester_id: member_id,
                household_id: household.id(),
            })
            .await
            .expect("Leaving household should succeed");

        let (_, events) = result.into_parts();

        let stored_member = household_repository
            .find_member(&household.id(), &member_id)
            .await
            .expect("Membership lookup should succeed");

        assert!(stored_member.is_none());

        assert_eq!(
            events,
            vec![AppEvent::Household(HouseholdApplicationEvent::MemberLeft {
                household_id: household.id(),
                member_id,
            })]
        );
    }

    #[tokio::test]
    async fn owner_cannot_leave_household() {
        let (service, household_repository) = build_leave_household_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let result = service
            .execute(LeaveHouseholdCommand {
                requester_id: owner_id,
                household_id: household.id(),
            })
            .await;

        assert!(matches!(result, Err(LeaveHouseholdError::OwnerCannotLeave)));

        let stored_owner = household_repository
            .find_member(&household.id(), &owner_id)
            .await
            .expect("Membership lookup should succeed");

        assert!(stored_owner.is_some());
    }

    #[tokio::test]
    async fn non_member_is_forbidden() {
        let (service, household_repository) = build_leave_household_service();

        let owner_id = UserId::new();
        let non_member_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let result = service
            .execute(LeaveHouseholdCommand {
                requester_id: non_member_id,
                household_id: household.id(),
            })
            .await;

        assert!(matches!(
            result,
            Err(LeaveHouseholdError::HouseholdAccess(
                HouseholdAccessError::Forbidden
            ))
        ));
    }

    #[tokio::test]
    async fn unknown_household_returns_not_found() {
        let (service, _) = build_leave_household_service();

        let result = service
            .execute(LeaveHouseholdCommand {
                requester_id: UserId::new(),
                household_id: HouseholdId::new(),
            })
            .await;

        assert!(matches!(
            result,
            Err(LeaveHouseholdError::HouseholdAccess(
                HouseholdAccessError::HouseholdNotFound
            ))
        ));
    }

    #[tokio::test]
    async fn repository_failure_returns_internal() {
        let inner_repository = Arc::new(InMemoryHouseholdRepository::new());

        let owner_id = UserId::new();
        let member_id = UserId::new();

        let (household, _) =
            insert_owned_household(&inner_repository, owner_id, HouseholdKind::Shared).await;

        insert_member(&inner_repository, household.id(), member_id).await;

        let policy = Arc::new(DefaultHouseholdAccessPolicy::new(inner_repository));

        let repository = Arc::new(FailingHouseholdRepository);

        let service = LeaveHouseholdService::new(repository, policy);

        let result = service
            .execute(LeaveHouseholdCommand {
                requester_id: member_id,
                household_id: household.id(),
            })
            .await;

        assert!(matches!(
            result,
            Err(LeaveHouseholdError::Internal(InternalError::Failed))
        ));
    }
}
