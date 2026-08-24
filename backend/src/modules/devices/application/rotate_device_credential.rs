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

pub struct RotateDeviceCredentialCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub device_id: DeviceId,
}

pub struct RotateDeviceCredentialService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    device_repository: Arc<dyn DeviceRepository>,
    device_credential_repository: Arc<dyn DeviceCredentialRepository>,
    token_generator: Arc<dyn TokenGenerator<DeviceToken>>,
    token_hasher: Arc<dyn TokenHasher<DeviceToken, DeviceTokenHash>>,
}

impl RotateDeviceCredentialService {
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
        command: RotateDeviceCredentialCommand,
    ) -> Result<DeviceToken, RotateDeviceCredentialError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await
            .map_err(map_household_access_error)?;

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
                RotateDeviceCredentialError::Internal(InternalError::Failed)
            })?
            .ok_or(RotateDeviceCredentialError::DeviceNotFound)?;

        if device.is_revoked() {
            return Err(RotateDeviceCredentialError::DeviceRevoked);
        }

        let token = self
            .token_generator
            .generate()
            .map_err(|_| RotateDeviceCredentialError::TokenGenerationFailed)?;

        let hash = self.token_hasher.hash(&token);

        let credential = DeviceCredential::new(
            DeviceCredentialId::new(),
            command.device_id,
            hash,
            Utc::now(),
        );

        self.device_credential_repository
            .rotate(&command.device_id, &credential, Utc::now())
            .await
            .map_err(|error| match error {
                DeviceCredentialRepositoryError::CredentialNotFound => {
                    RotateDeviceCredentialError::CredentialNotFound
                }
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        device_id = %command.device_id,
                        "Failed to rotate device credential"
                    );
                    RotateDeviceCredentialError::Internal(InternalError::Failed)
                }
            })?;

        Ok(token)
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RotateDeviceCredentialError {
    #[error("Device was not found")]
    DeviceNotFound,
    #[error("Device is revoked")]
    DeviceRevoked,
    #[error("Device has no active credential")]
    CredentialNotFound,
    #[error("Device token generation failed")]
    TokenGenerationFailed,
    #[error("You do not have permission")]
    Forbidden,
    #[error("Household was not found")]
    HouseholdNotFound,
    #[error(transparent)]
    Internal(#[from] InternalError),
}

fn map_household_access_error(error: HouseholdAccessError) -> RotateDeviceCredentialError {
    match error {
        HouseholdAccessError::Forbidden => RotateDeviceCredentialError::Forbidden,
        HouseholdAccessError::HouseholdNotFound => RotateDeviceCredentialError::HouseholdNotFound,
        HouseholdAccessError::Internal(error) => RotateDeviceCredentialError::Internal(error),
    }
}

//TODO: Write tests
/*
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn household_member_can_rotate_device_credential() {}

    #[tokio::test]
    async fn rotated_token_replaces_previous_active_credential() {}

    #[tokio::test]
    async fn device_without_active_credential_cannot_be_rotated() {}

    #[tokio::test]
    async fn revoked_device_cannot_rotate_credential() {}

    #[tokio::test]
    async fn unknown_device_returns_not_found() {}

    #[tokio::test]
    async fn non_member_is_forbidden() {}
}
*/
