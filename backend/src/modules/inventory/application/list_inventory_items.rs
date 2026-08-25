use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
        inventory::{ports::InventoryItemQuery, read_models::InventoryItemListEntry},
    },
    shared::application::InternalError,
};

pub struct ListInventoryItemsCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
}

pub struct ListInventoryItemsService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    inventory_item_query: Arc<dyn InventoryItemQuery>,
}

impl ListInventoryItemsService {
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
        command: ListInventoryItemsCommand,
    ) -> Result<Vec<InventoryItemListEntry>, ListInventoryItemsError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        self.inventory_item_query
            .find_active_for_household(&command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to list active inventory items",
                );
                ListInventoryItemsError::Internal(InternalError::Failed)
            })
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ListInventoryItemsError {
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
    async fn household_member_can_list_active_inventory_items() {}

    #[tokio::test]
    async fn non_member_is_forbidden() {}

    #[tokio::test]
    async fn query_failure_returns_internal() {}
}
*/
