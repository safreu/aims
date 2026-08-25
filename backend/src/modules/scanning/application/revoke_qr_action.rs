use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
        scanning::{
            domain::QrActionId,
            ports::{QrActionRepository, QrActionRepositoryError},
        },
    },
    shared::application::InternalError,
};

pub struct RevokeQrActionCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub qr_action_id: QrActionId,
}

pub struct RevokeQrActionService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    qr_action_repository: Arc<dyn QrActionRepository>,
}

impl RevokeQrActionService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        qr_action_repository: Arc<dyn QrActionRepository>,
    ) -> Self {
        Self {
            household_access_policy,
            qr_action_repository,
        }
    }

    pub async fn execute(&self, command: RevokeQrActionCommand) -> Result<(), RevokeQrActionError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let mut action = self
            .qr_action_repository
            .find_by_id_for_household(&command.qr_action_id, &command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    qr_action_id = %command.qr_action_id,
                    "Failed to locate QR action for household"
                );
                RevokeQrActionError::Internal(InternalError::Failed)
            })?
            .ok_or(RevokeQrActionError::QrActionNotFound)?;

        action
            .revoke(Utc::now())
            .map_err(|_| RevokeQrActionError::AlreadyRevoked)?;

        self.qr_action_repository
            .revoke(&action)
            .await
            .map_err(|error| match error {
                QrActionRepositoryError::QrActionNotFound => RevokeQrActionError::QrActionNotFound,
                QrActionRepositoryError::QrActionRevoked => RevokeQrActionError::AlreadyRevoked,
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        qr_action_id = %command.qr_action_id,
                        "Failed to persist revocation of QR action"
                    );
                    RevokeQrActionError::Internal(InternalError::Failed)
                }
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RevokeQrActionError {
    #[error("QR action not found")]
    QrActionNotFound,
    #[error("QR action already revoked")]
    AlreadyRevoked,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

//TODO: Write tests
/*
#[cfg(test)]
mod tests {
    use super::*;

}
*/
