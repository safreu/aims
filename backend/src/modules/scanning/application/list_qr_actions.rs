use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
        scanning::{domain::QrAction, ports::QrActionRepository},
    },
    shared::application::InternalError,
};

pub struct ListQrActionsCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
}

pub struct ListQrActionsService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    qr_action_repository: Arc<dyn QrActionRepository>,
}

impl ListQrActionsService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        qr_action_repository: Arc<dyn QrActionRepository>,
    ) -> Self {
        Self {
            household_access_policy,
            qr_action_repository,
        }
    }

    pub async fn execute(
        &self,
        command: ListQrActionsCommand,
    ) -> Result<Vec<QrAction>, ListQrActionsError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let actions = self
            .qr_action_repository
            .find_active_for_household(&command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to locate active QR actions for household"
                );
                ListQrActionsError::Internal(InternalError::Failed)
            })?;

        Ok(actions)
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ListQrActionsError {
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

//TODO: Write these tests
/*
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn member_can_list_qr_actions() {}

    #[tokio::test]
    async fn only_qr_actions_for_requested_household_are_returned() {}

    #[tokio::test]
    async fn non_member_cannot_list_qr_actions() {}

    #[tokio::test]
    async fn repository_failure_returns_internal_error() {}
}
*/
