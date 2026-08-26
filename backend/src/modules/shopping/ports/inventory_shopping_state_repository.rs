use async_trait::async_trait;

use crate::{
    modules::{
        households::domain::HouseholdId, inventory::domain::InventoryItemId,
        shopping::domain::InventoryShoppingState,
    },
    shared::db::PersistenceError,
};

#[async_trait]
pub trait InventoryShoppingStateRepository: Send + Sync {
    async fn find_by_item(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
    ) -> Result<Option<InventoryShoppingState>, InventoryShoppingStateRepositoryError>;

    async fn upsert(
        &self,
        state: &InventoryShoppingState,
    ) -> Result<(), InventoryShoppingStateRepositoryError>;

    async fn delete(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
    ) -> Result<(), InventoryShoppingStateRepositoryError>;
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum InventoryShoppingStateRepositoryError {
    #[error("Invalid stored shopping data")]
    InvalidStoredData,
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}
