use crate::{modules::devices::domain::Device, shared::db::PersistenceError};
use async_trait::async_trait;

#[async_trait]
pub trait DeviceRevocationRepository: Send + Sync {
    async fn revoke(&self, device: &Device) -> Result<(), DeviceRevocationRepositoryError>;
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DeviceRevocationRepositoryError {
    #[error("Device was not found")]
    DeviceNotFound,
    #[error("Device is already revoked")]
    DeviceRevoked,
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}
