use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    modules::{
        households::domain::HouseholdId,
        inventory::{
            adapters::InMemoryInventoryItemRepository,
            domain::{InventoryItem, InventoryItemError, InventoryItemId},
            ports::{
                InventoryItemRepository, InventoryStockRepository, InventoryStockRepositoryError,
                StockMutationContext,
            },
        },
    },
    shared::db::PersistenceError,
};

pub struct InMemoryInventoryStockRepository {
    inventory_item_repository: Arc<InMemoryInventoryItemRepository>,
}

impl InMemoryInventoryStockRepository {
    pub fn new(inventory_item_repository: Arc<InMemoryInventoryItemRepository>) -> Self {
        Self {
            inventory_item_repository,
        }
    }

    async fn save_item(&self, item: &InventoryItem) -> Result<(), InventoryStockRepositoryError> {
        self.inventory_item_repository
            .update(item)
            .await
            .map_err(|_| InventoryStockRepositoryError::Persistence(PersistenceError::Failed))
    }

    async fn find_item(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
    ) -> Result<InventoryItem, InventoryStockRepositoryError> {
        self.inventory_item_repository
            .find_by_id(item_id, household_id)
            .await
            .map_err(|_| InventoryStockRepositoryError::Persistence(PersistenceError::Failed))?
            .ok_or(InventoryStockRepositoryError::ItemNotFound)
    }
}

#[async_trait]
impl InventoryStockRepository for InMemoryInventoryStockRepository {
    async fn increase(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
        amount: u32,
        _context: &StockMutationContext,
        now: DateTime<Utc>,
    ) -> Result<(), InventoryStockRepositoryError> {
        let mut item = self.find_item(household_id, item_id).await?;

        item.increase(amount, now)
            .map_err(map_inventory_item_error)?;

        self.save_item(&item).await
    }

    async fn decrease(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
        amount: u32,
        _context: &StockMutationContext,
        now: DateTime<Utc>,
    ) -> Result<(), InventoryStockRepositoryError> {
        let mut item = self.find_item(household_id, item_id).await?;

        item.decrease(amount, now)
            .map_err(map_inventory_item_error)?;

        self.save_item(&item).await
    }

    async fn set(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
        stock: u32,
        _context: &StockMutationContext,
        now: DateTime<Utc>,
    ) -> Result<(), InventoryStockRepositoryError> {
        let mut item = self.find_item(household_id, item_id).await?;

        item.set_stock(stock, now)
            .map_err(map_inventory_item_error)?;

        self.save_item(&item).await
    }
}

fn map_inventory_item_error(error: InventoryItemError) -> InventoryStockRepositoryError {
    match error {
        InventoryItemError::Archived => InventoryStockRepositoryError::ItemArchived,
        InventoryItemError::InsufficientStock => InventoryStockRepositoryError::InsufficientStock,
        InventoryItemError::StockOverflow => InventoryStockRepositoryError::StockOverflow,
        InventoryItemError::AlreadyArchived | InventoryItemError::NotArchived => {
            InventoryStockRepositoryError::Persistence(PersistenceError::Failed)
        }
    }
}
