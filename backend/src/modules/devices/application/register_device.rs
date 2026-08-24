use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        accounts::domain::UserId,
        devices::{
            domain::{Device, DeviceId, DeviceKind, DeviceName},
            ports::DeviceRepository,
        },
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
    },
    shared::application::InternalError,
};

pub struct RegisterDeviceCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub name: String,
    pub kind: String,
}

pub struct RegisterDeviceService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    device_repository: Arc<dyn DeviceRepository>,
}

impl RegisterDeviceService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        device_repository: Arc<dyn DeviceRepository>,
    ) -> Self {
        Self {
            household_access_policy,
            device_repository,
        }
    }

    pub async fn execute(
        &self,
        command: RegisterDeviceCommand,
    ) -> Result<DeviceId, RegisterDeviceError> {
        let name =
            DeviceName::parse(&command.name).map_err(|_| RegisterDeviceError::InvalidName)?;

        let kind =
            DeviceKind::parse(&command.kind).map_err(|_| RegisterDeviceError::InvalidKind)?;

        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await
            .map_err(map_household_access_error)?;

        let now = Utc::now();
        let device_id = DeviceId::new();

        let device = Device::new(device_id, command.household_id, name, kind, now, now);

        self.device_repository
            .insert(&device)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    device_id = %device_id,
                    "Failed to register device"
                );

                RegisterDeviceError::Internal(InternalError::Failed)
            })?;

        Ok(device_id)
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RegisterDeviceError {
    #[error("The device name is invalid")]
    InvalidName,
    #[error("The device kind is invalid")]
    InvalidKind,
    #[error("You do not have permission")]
    Forbidden,
    #[error("Household was not found")]
    HouseholdNotFound,
    #[error(transparent)]
    Internal(#[from] InternalError),
}

fn map_household_access_error(error: HouseholdAccessError) -> RegisterDeviceError {
    match error {
        HouseholdAccessError::Forbidden => RegisterDeviceError::Forbidden,
        HouseholdAccessError::HouseholdNotFound => RegisterDeviceError::HouseholdNotFound,
        HouseholdAccessError::Internal(error) => RegisterDeviceError::Internal(error),
    }
}

//TODO: Implement in memory representation of device_repository and write these tests
/*
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn household_owner_can_register_device() {}

    #[tokio::test]
    async fn non_owner_is_forbidden() {}

    #[tokio::test]
    async fn invalid_device_name_is_rejected() {}

    #[tokio::test]
    async fn invalid_device_kind_is_rejected() {}
}
*/
