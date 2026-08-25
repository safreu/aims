use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        accounts::domain::UserId,
        devices::{
            domain::{DeviceError, DeviceId},
            ports::{
                DeviceRepository, DeviceRevocationRepository, DeviceRevocationRepositoryError,
            },
        },
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
    },
    shared::application::InternalError,
};

pub struct RevokeDeviceCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub device_id: DeviceId,
}

pub struct RevokeDeviceService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    device_repository: Arc<dyn DeviceRepository>,
    device_revocation_repository: Arc<dyn DeviceRevocationRepository>,
}

impl RevokeDeviceService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        device_repository: Arc<dyn DeviceRepository>,
        device_revocation_repository: Arc<dyn DeviceRevocationRepository>,
    ) -> Self {
        Self {
            household_access_policy,
            device_repository,
            device_revocation_repository,
        }
    }

    pub async fn execute(&self, command: RevokeDeviceCommand) -> Result<(), RevokeDeviceError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let mut device = self
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
                RevokeDeviceError::Internal(InternalError::Failed)
            })?
            .ok_or(RevokeDeviceError::DeviceNotFound)?;

        let now = Utc::now();

        device.revoke(now).map_err(|error| match error {
            DeviceError::Revoked => RevokeDeviceError::AlreadyRevoked,
            other => {
                tracing::error!(
                    error = ?other,
                    household_id = %command.household_id,
                    device_id = %command.device_id,
                 "Unexpected device error while revoking device",
                );
                RevokeDeviceError::Internal(InternalError::Failed)
            }
        })?;

        self.device_revocation_repository
            .revoke(&device)
            .await
            .map_err(|error| match error {
                DeviceRevocationRepositoryError::DeviceNotFound => {
                    RevokeDeviceError::DeviceNotFound
                }
                DeviceRevocationRepositoryError::DeviceRevoked => RevokeDeviceError::AlreadyRevoked,
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        device_id = %command.device_id,
                        "Failed to persist device revocation"
                    );
                    RevokeDeviceError::Internal(InternalError::Failed)
                }
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RevokeDeviceError {
    #[error("Device was not found")]
    DeviceNotFound,
    #[error("Device is revoked")]
    AlreadyRevoked,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

//TODO: Implement in memory representation of device_repository and write these tests
/*
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn household_member_can_revoke_device() {}

    #[tokio::test]
    async fn unknown_device_returns_not_found() {}

    #[tokio::test]
    async fn already_revoked_device_is_rejected() {}

    #[tokio::test]
    async fn non_member_is_forbidden() {}
}
*/
