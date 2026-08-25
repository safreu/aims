use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        devices::{domain::Device, ports::DeviceRepository},
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
    },
    shared::application::InternalError,
};

pub struct ListDevicesCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
}

pub struct ListDevicesService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    device_repository: Arc<dyn DeviceRepository>,
}

impl ListDevicesService {
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
        command: ListDevicesCommand,
    ) -> Result<Vec<Device>, ListDevicesError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let devices = self
            .device_repository
            .find_active_for_household(&command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to list active devices"
                );

                ListDevicesError::Internal(InternalError::Failed)
            })?;

        Ok(devices)
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ListDevicesError {
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
    async fn household_member_can_list_active_devices() {}

    #[tokio::test]
    async fn revoked_devices_are_not_returned() {}

    #[tokio::test]
    async fn non_member_is_forbidden() {}
}
*/
