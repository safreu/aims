use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
        inventory::{
            domain::InventoryItemId,
            ports::{InventoryItemRepository, InventoryStockHistoryQuery},
            read_models::InventoryStockHistoryEntry,
        },
    },
    shared::application::InternalError,
};

pub struct ListInventoryStockHistoryCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub item_id: InventoryItemId,
}

pub struct ListInventoryStockHistoryService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    inventory_item_repository: Arc<dyn InventoryItemRepository>,
    history_query: Arc<dyn InventoryStockHistoryQuery>,
}

impl ListInventoryStockHistoryService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        inventory_item_repository: Arc<dyn InventoryItemRepository>,
        history_query: Arc<dyn InventoryStockHistoryQuery>,
    ) -> Self {
        Self {
            household_access_policy,
            inventory_item_repository,
            history_query,
        }
    }

    pub async fn execute(
        &self,
        command: ListInventoryStockHistoryCommand,
    ) -> Result<Vec<InventoryStockHistoryEntry>, ListInventoryStockHistoryError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let item = self
            .inventory_item_repository
            .find_by_id(&command.item_id, &command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    item_id = %command.item_id,
                    "Failed to check inventory item existence",
                );
                ListInventoryStockHistoryError::Internal(InternalError::Failed)
            })?;

        if item.is_none() {
            return Err(ListInventoryStockHistoryError::ItemNotFound);
        }

        self.history_query
            .find_for_item(&command.household_id, &command.item_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    item_id = %command.item_id,
                    "Failed to load inventory stock history"
                );
                ListInventoryStockHistoryError::Internal(InternalError::Failed)
            })
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ListInventoryStockHistoryError {
    #[error("Inventory item was not found")]
    ItemNotFound,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

//TODO: Implement the in memory representation of the adapter and implement these tests
/*
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn household_member_can_list_inventory_stock_history() {}

    #[tokio::test]
    async fn non_member_is_forbidden() {}
}
*/
