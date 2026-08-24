use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    modules::devices::domain::{DeviceCredential, DeviceId, DeviceTokenHash},
    shared::db::PersistenceError,
};

#[async_trait]
pub trait DeviceCredentialRepository: Send + Sync {
    async fn insert(
        &self,
        credential: &DeviceCredential,
    ) -> Result<(), DeviceCredentialRepositoryError>;

    async fn find_active_by_device_id(
        &self,
        device_id: &DeviceId,
    ) -> Result<Option<DeviceCredential>, DeviceCredentialRepositoryError>;

    async fn find_active_by_token_hash(
        &self,
        token_hash: &DeviceTokenHash,
    ) -> Result<Option<DeviceCredential>, DeviceCredentialRepositoryError>;

    async fn revoke_active(
        &self,
        device_id: &DeviceId,
        now: DateTime<Utc>,
    ) -> Result<(), DeviceCredentialRepositoryError>;
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DeviceCredentialRepositoryError {
    #[error("Device credentials was not found")]
    CredentialNotFound,
    #[error("Device already has an active credential")]
    ActiveCredentialAlreadyExists,
    #[error("Invalid stored device credential")]
    InvalidStoredData,
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}
