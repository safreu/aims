use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::{HouseholdEvent, HouseholdId, HouseholdRole},
            ports::{
                HouseholdAccessError, HouseholdAccessPolicy, HouseholdEventPublisher,
                HouseholdRepository, HouseholdRepositoryError,
            },
        },
    },
    shared::application::InternalError,
};

pub struct LeaveHouseholdCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
}

pub struct LeaveHouseholdService {
    household_repository: Arc<dyn HouseholdRepository>,
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    household_events_publisher: Arc<dyn HouseholdEventPublisher>,
}

impl LeaveHouseholdService {
    pub fn new(
        household_repository: Arc<dyn HouseholdRepository>,
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        household_events_publisher: Arc<dyn HouseholdEventPublisher>,
    ) -> Self {
        Self {
            household_repository,
            household_access_policy,
            household_events_publisher,
        }
    }

    pub async fn execute(&self, command: LeaveHouseholdCommand) -> Result<(), LeaveHouseholdError> {
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

        self.household_events_publisher
            .publish(command.household_id, HouseholdEvent::HouseholdChanged)
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to publish household changed event"
                );
                LeaveHouseholdError::Internal(InternalError::Failed)
            })?;
        Ok(())
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
