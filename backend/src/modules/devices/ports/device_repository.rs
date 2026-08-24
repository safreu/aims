use async_trait::async_trait;

use crate::{
    modules::{
        devices::domain::{Device, DeviceId},
        households::domain::HouseholdId,
    },
    shared::db::PersistenceError,
};

#[async_trait]
pub trait DeviceRepository: Send + Sync {
    async fn insert(&self, device: &Device) -> Result<(), DeviceRepositoryError>;

    async fn find_by_id_for_household(
        &self,
        device_id: &DeviceId,
        household_id: &HouseholdId,
    ) -> Result<Option<Device>, DeviceRepositoryError>;

    async fn find_active_for_household(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<Device>, DeviceRepositoryError>;

    async fn update(&self, device: &Device) -> Result<(), DeviceRepositoryError>;

    async fn find_by_id(
        &self,
        device_id: &DeviceId,
    ) -> Result<Option<Device>, DeviceRepositoryError>;
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DeviceRepositoryError {
    #[error("Device was not found")]
    DeviceNotFound,
    #[error("Device revoked")]
    DeviceRevoked,
    #[error("Invalid stored device data")]
    InvalidStoredData,
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}
