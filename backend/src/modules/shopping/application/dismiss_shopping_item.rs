use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
        inventory::{domain::InventoryItemId, ports::InventoryItemRepository},
        shopping::{domain::InventoryShoppingState, ports::InventoryShoppingStateRepository},
    },
    shared::application::InternalError,
};

pub struct DismissShoppingItemCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub item_id: InventoryItemId,
}

pub struct DismissShoppingItemService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    inventory_item_repository: Arc<dyn InventoryItemRepository>,
    shopping_state_repository: Arc<dyn InventoryShoppingStateRepository>,
}

impl DismissShoppingItemService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        inventory_item_repository: Arc<dyn InventoryItemRepository>,
        shopping_state_repository: Arc<dyn InventoryShoppingStateRepository>,
    ) -> Self {
        Self {
            household_access_policy,
            inventory_item_repository,
            shopping_state_repository,
        }
    }

    pub async fn execute(
        &self,
        command: DismissShoppingItemCommand,
    ) -> Result<(), DismissShoppingItemError> {
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
                    "Failed to load item for household"
                );
                DismissShoppingItemError::Internal(InternalError::Failed)
            })?
            .ok_or(DismissShoppingItemError::ItemNotFound)?;

        if item.archived_at().is_some() {
            return Err(DismissShoppingItemError::ItemArchived);
        }

        let mut state = self
            .shopping_state_repository
            .find_by_item(&command.household_id, &command.item_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    item_id = %command.item_id,
                    "Failed to load shopping state for item"
                );
                DismissShoppingItemError::Internal(InternalError::Failed)
            })?
            .unwrap_or_else(|| InventoryShoppingState::new(command.household_id, command.item_id));

        state.dismiss();

        self.shopping_state_repository
            .upsert(&state)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    item_id = %command.item_id,
                    "Failed to persist shopping state"
                );
                DismissShoppingItemError::Internal(InternalError::Failed)
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DismissShoppingItemError {
    #[error("Inventory item not found")]
    ItemNotFound,
    #[error("Inventory item is archived")]
    ItemArchived,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

//TODO: Write tests
/*
#[cfg(test)]
mod tests {
    use super::*;

}
*/
