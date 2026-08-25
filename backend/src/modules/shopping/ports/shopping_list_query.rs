use async_trait::async_trait;

use crate::{
    modules::{households::domain::HouseholdId, shopping::read_models::InventoryShoppingEntry},
    shared::db::PersistenceError,
};

#[async_trait]
pub trait ShoppingListQuery: Send + Sync {
    async fn list_inventory_entries(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<InventoryShoppingEntry>, ShoppingListQueryError>;
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ShoppingListQueryError {
    #[error("Invalid stored shopping data")]
    InvalidStoredData,
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}
