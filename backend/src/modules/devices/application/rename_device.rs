use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        accounts::domain::UserId,
        devices::{
            domain::{DeviceId, DeviceName},
            ports::{DeviceRepository, DeviceRepositoryError},
        },
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
    },
    shared::application::InternalError,
};

pub struct RenameDeviceCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub device_id: DeviceId,
    pub name: String,
}

pub struct RenameDeviceService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    device_repository: Arc<dyn DeviceRepository>,
}

impl RenameDeviceService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        device_repository: Arc<dyn DeviceRepository>,
    ) -> Self {
        Self {
            household_access_policy,
            device_repository,
        }
    }

    pub async fn execute(&self, command: RenameDeviceCommand) -> Result<(), RenameDeviceError> {
        let name = DeviceName::parse(&command.name).map_err(|_| RenameDeviceError::InvalidName)?;

        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await
            .map_err(map_household_access_error)?;

        let mut device = self
            .device_repository
            .find_by_id(&command.device_id, &command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    device_id = %command.device_id,
                    "Failed to load device"
                );
                RenameDeviceError::Internal(InternalError::Failed)
            })?
            .ok_or(RenameDeviceError::DeviceNotFound)?;

        let now = Utc::now();

        device
            .rename(name, now)
            .map_err(|_| RenameDeviceError::DeviceRevoked)?;

        self.device_repository
            .update(&device)
            .await
            .map_err(|error| match error {
                DeviceRepositoryError::DeviceNotFound => RenameDeviceError::DeviceNotFound,
                DeviceRepositoryError::DeviceRevoked => RenameDeviceError::DeviceRevoked,
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        device_id = %command.device_id,
                        "Failed to rename device"
                    );
                    RenameDeviceError::Internal(InternalError::Failed)
                }
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RenameDeviceError {
    #[error("The device name is invalid")]
    InvalidName,
    #[error("Device was not found")]
    DeviceNotFound,
    #[error("Device is revoked")]
    DeviceRevoked,
    #[error("You do not have permission")]
    Forbidden,
    #[error("Household was not found")]
    HouseholdNotFound,
    #[error(transparent)]
    Internal(#[from] InternalError),
}

fn map_household_access_error(error: HouseholdAccessError) -> RenameDeviceError {
    match error {
        HouseholdAccessError::Forbidden => RenameDeviceError::Forbidden,
        HouseholdAccessError::HouseholdNotFound => RenameDeviceError::HouseholdNotFound,
        HouseholdAccessError::Internal(error) => RenameDeviceError::Internal(error),
    }
}

//TODO: Implement in memory representation of device_repository and write these tests
/*
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn household_member_can_rename_device() {}

    #[tokio::test]
    async fn invalid_device_name_is_rejected() {}

    #[tokio::test]
    async fn unknown_device_returns_not_found() {}

    #[tokio::test]
    async fn revoked_device_cannot_be_renamed() {}

    #[tokio::test]
    async fn non_member_is_forbidden() {}
}
*/
