use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    modules::{
        households::domain::HouseholdId,
        inventory::{domain::InventoryItemId, ports::StockMutationContext},
    },
    shared::db::PersistenceError,
};

#[async_trait]
pub trait InventoryStockRepository: Sync + Send {
    async fn increase(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
        amount: u32,
        context: &StockMutationContext,
        now: DateTime<Utc>,
    ) -> Result<(), InventoryStockRepositoryError>;

    async fn decrease(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
        amount: u32,
        context: &StockMutationContext,
        now: DateTime<Utc>,
    ) -> Result<(), InventoryStockRepositoryError>;

    async fn set(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
        amount: u32,
        context: &StockMutationContext,
        now: DateTime<Utc>,
    ) -> Result<(), InventoryStockRepositoryError>;
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum InventoryStockRepositoryError {
    #[error("Inventory item was not found")]
    ItemNotFound,
    #[error("Inventory item is archived")]
    ItemArchived,
    #[error("Insufficient stock")]
    InsufficientStock,
    #[error("Stock value overflow")]
    StockOverflow,
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}
