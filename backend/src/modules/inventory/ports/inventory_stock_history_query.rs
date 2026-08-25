use async_trait::async_trait;

use crate::modules::households::domain::HouseholdId;
use crate::modules::inventory::domain::InventoryItemId;
use crate::modules::inventory::read_models::InventoryStockHistoryEntry;
use crate::shared::db::PersistenceError;

#[async_trait]
pub trait InventoryStockHistoryQuery: Send + Sync {
    async fn find_for_item(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
    ) -> Result<Vec<InventoryStockHistoryEntry>, InventoryStockHistoryQueryError>;
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum InventoryStockHistoryQueryError {
    #[error("Invalid stored data")]
    InvalidStoredData,
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}
