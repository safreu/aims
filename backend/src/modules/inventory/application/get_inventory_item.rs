use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
        inventory::{
            domain::InventoryItemId, ports::InventoryItemQuery, read_models::InventoryItemListEntry,
        },
    },
    shared::application::InternalError,
};

pub struct GetInventoryItemCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub item_id: InventoryItemId,
}

pub struct GetInventoryItemService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    inventory_item_query: Arc<dyn InventoryItemQuery>,
}

impl GetInventoryItemService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        inventory_item_query: Arc<dyn InventoryItemQuery>,
    ) -> Self {
        Self {
            household_access_policy,
            inventory_item_query,
        }
    }

    pub async fn execute(
        &self,
        command: GetInventoryItemCommand,
    ) -> Result<InventoryItemListEntry, GetInventoryItemError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        self.inventory_item_query
            .find_active_by_id(&command.household_id, &command.item_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    item_id = %command.item_id,
                    "Failed to load inventory item"
                );
                GetInventoryItemError::Internal(InternalError::Failed)
            })?
            .ok_or(GetInventoryItemError::ItemNotFound)
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum GetInventoryItemError {
    #[error("Inventory item was not found")]
    ItemNotFound,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

//TODO: Implement the in memory representation of the inventory_item_query and write the following tests
/*
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn household_member_can_get_inventory_item() {}

    #[tokio::test]
    async fn unknown_inventory_item_returns_not_found() {}

    #[tokio::test]
    async fn non_member_is_forbidden() {}
}
 */
