use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        accounts::domain::UserId,
        devices::{
            domain::{
                DeviceCredential, DeviceCredentialId, DeviceId, DeviceToken, DeviceTokenHash,
            },
            ports::{
                DeviceCredentialRepository, DeviceCredentialRepositoryError, DeviceRepository,
            },
        },
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
    },
    shared::{
        application::InternalError,
        auth::{TokenGenerator, TokenHasher},
    },
};

pub struct IssueDeviceCredentialCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub device_id: DeviceId,
}

pub struct IssueDeviceCredentialService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    device_repository: Arc<dyn DeviceRepository>,
    device_credential_repository: Arc<dyn DeviceCredentialRepository>,
    token_generator: Arc<dyn TokenGenerator<DeviceToken>>,
    token_hasher: Arc<dyn TokenHasher<DeviceToken, DeviceTokenHash>>,
}

impl IssueDeviceCredentialService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        device_repository: Arc<dyn DeviceRepository>,
        device_credential_repository: Arc<dyn DeviceCredentialRepository>,
        token_generator: Arc<dyn TokenGenerator<DeviceToken>>,
        token_hasher: Arc<dyn TokenHasher<DeviceToken, DeviceTokenHash>>,
    ) -> Self {
        Self {
            household_access_policy,
            device_repository,
            device_credential_repository,
            token_generator,
            token_hasher,
        }
    }

    pub async fn execute(
        &self,
        command: IssueDeviceCredentialCommand,
    ) -> Result<DeviceToken, IssueDeviceCredentialError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let device = self
            .device_repository
            .find_by_id_for_household(&command.device_id, &command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    device_id = %command.device_id,
                    "Failed to load device"
                );
                IssueDeviceCredentialError::Internal(InternalError::Failed)
            })?
            .ok_or(IssueDeviceCredentialError::DeviceNotFound)?;

        if device.is_revoked() {
            return Err(IssueDeviceCredentialError::DeviceRevoked);
        }

        let token = self
            .token_generator
            .generate()
            .map_err(|_| IssueDeviceCredentialError::TokenGenerationFailed)?;

        let hash = self.token_hasher.hash(&token);

        let credential = DeviceCredential::new(
            DeviceCredentialId::new(),
            command.device_id,
            hash,
            Utc::now(),
        );

        self.device_credential_repository
            .insert(&credential)
            .await
            .map_err(|error| match error {
                DeviceCredentialRepositoryError::ActiveCredentialAlreadyExists => {
                    IssueDeviceCredentialError::ActiveCredentialAlreadyExists
                }
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        device_id = %command.device_id,
                        "Failed to persist device credential"
                    );
                    IssueDeviceCredentialError::Internal(InternalError::Failed)
                }
            })?;

        Ok(token)
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum IssueDeviceCredentialError {
    #[error("Device was not found")]
    DeviceNotFound,
    #[error("Device is revoked")]
    DeviceRevoked,
    #[error("An active credential for this device already exists")]
    ActiveCredentialAlreadyExists,
    #[error("Device token generation failed")]
    TokenGenerationFailed,
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
    async fn household_member_can_issue_device_credential() {}

    #[tokio::test]
    async fn issued_token_is_not_stored_in_plaintext() {}

    #[tokio::test]
    async fn second_active_credential_is_rejected() {}

    #[tokio::test]
    async fn revoked_device_cannot_receive_credential() {}

    #[tokio::test]
    async fn unknown_device_returns_not_found() {}

    #[tokio::test]
    async fn non_member_is_forbidden() {}
}
*/
