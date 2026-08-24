use std::sync::Arc;

use crate::{
    modules::{
        devices::{
            domain::{DeviceId, DeviceToken, DeviceTokenHash},
            ports::{DeviceCredentialRepository, DeviceRepository},
        },
        households::domain::HouseholdId,
    },
    shared::{application::InternalError, auth::TokenHasher},
};

pub struct AuthenticatedDevice {
    pub device_id: DeviceId,
    pub household_id: HouseholdId,
}

pub struct AuthenticateDeviceCommand {
    pub token: DeviceToken,
}

pub struct AuthenticateDeviceService {
    device_repository: Arc<dyn DeviceRepository>,
    device_credential_repository: Arc<dyn DeviceCredentialRepository>,
    token_hasher: Arc<dyn TokenHasher<DeviceToken, DeviceTokenHash>>,
}

impl AuthenticateDeviceService {
    pub fn new(
        device_repository: Arc<dyn DeviceRepository>,
        device_credential_repository: Arc<dyn DeviceCredentialRepository>,
        token_hasher: Arc<dyn TokenHasher<DeviceToken, DeviceTokenHash>>,
    ) -> Self {
        Self {
            device_repository,
            device_credential_repository,
            token_hasher,
        }
    }

    pub async fn execute(
        &self,
        command: AuthenticateDeviceCommand,
    ) -> Result<AuthenticatedDevice, AuthenticateDeviceError> {
        let hash = self.token_hasher.hash(&command.token);

        let credential = self
            .device_credential_repository
            .find_active_by_token_hash(&hash)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    "Failed to load device credential",
                );
                AuthenticateDeviceError::Internal(InternalError::Failed)
            })?
            .ok_or(AuthenticateDeviceError::InvalidCredentials)?;

        let device = self
            .device_repository
            .find_by_id(&credential.device_id())
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    device_id = %credential.device_id(),
                    "Failed to load authenticated device",
                );
                AuthenticateDeviceError::Internal(InternalError::Failed)
            })?
            .ok_or(AuthenticateDeviceError::InvalidCredentials)?;

        if device.is_revoked() {
            return Err(AuthenticateDeviceError::InvalidCredentials);
        }

        Ok(AuthenticatedDevice {
            device_id: device.id(),
            household_id: device.household_id(),
        })
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum AuthenticateDeviceError {
    #[error("The credentials are invalid")]
    InvalidCredentials,
    #[error(transparent)]
    Internal(#[from] InternalError),
}
