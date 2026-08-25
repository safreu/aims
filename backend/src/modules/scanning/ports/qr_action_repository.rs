use async_trait::async_trait;

use crate::{
    modules::{
        households::domain::HouseholdId,
        scanning::domain::{QrAction, QrActionId},
    },
    shared::db::PersistenceError,
};

#[async_trait]
pub trait QrActionRepository: Send + Sync {
    async fn insert(&self, action: &QrAction) -> Result<(), QrActionRepositoryError>;

    async fn find_by_id(
        &self,
        action_id: &QrActionId,
    ) -> Result<Option<QrAction>, QrActionRepositoryError>;

    async fn find_active_for_household(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<QrAction>, QrActionRepositoryError>;

    async fn revoke(&self, action: &QrAction) -> Result<(), QrActionRepositoryError>;
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum QrActionRepositoryError {
    #[error("QR action was not found")]
    QrActionNotFound,
    #[error("QR action is already revoked")]
    QrActionRevoked,
    #[error("Invalid stored QR action data")]
    InvalidStoredData,
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}
