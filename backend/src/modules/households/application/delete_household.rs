use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::{HouseholdEvent, HouseholdId},
            ports::{
                HouseholdAccessError, HouseholdAccessPolicy, HouseholdEventPublisher,
                HouseholdRepository,
            },
        },
    },
    shared::application::InternalError,
};

pub struct DeleteHouseholdCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
}

pub struct DeleteHouseholdService {
    household_repository: Arc<dyn HouseholdRepository>,
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    household_events_publisher: Arc<dyn HouseholdEventPublisher>,
}

impl DeleteHouseholdService {
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

    pub async fn execute(
        &self,
        command: DeleteHouseholdCommand,
    ) -> Result<(), DeleteHouseholdError> {
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

        self.household_events_publisher
            .publish(command.household_id, HouseholdEvent::HouseholdChanged)
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to publish household changed event"
                );
                DeleteHouseholdError::Internal(InternalError::Failed)
            })?;
        Ok(())
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
