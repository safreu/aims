use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::{HouseholdEvent, HouseholdId},
            ports::{HouseholdAccessError, HouseholdAccessPolicy, HouseholdEventPublisher},
        },
        inventory::{domain::InventoryItemId, ports::InventoryItemRepository},
        shopping::{domain::InventoryShoppingState, ports::InventoryShoppingStateRepository},
    },
    shared::application::InternalError,
};

pub struct SetCheckedCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub item_id: InventoryItemId,
    pub checked: bool,
}

pub struct SetCheckedService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    inventory_item_repository: Arc<dyn InventoryItemRepository>,
    shopping_state_repository: Arc<dyn InventoryShoppingStateRepository>,
    household_events_publisher: Arc<dyn HouseholdEventPublisher>,
}

impl SetCheckedService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        inventory_item_repository: Arc<dyn InventoryItemRepository>,
        shopping_state_repository: Arc<dyn InventoryShoppingStateRepository>,
        household_events_publisher: Arc<dyn HouseholdEventPublisher>,
    ) -> Self {
        Self {
            household_access_policy,
            inventory_item_repository,
            shopping_state_repository,
            household_events_publisher,
        }
    }

    pub async fn execute(&self, command: SetCheckedCommand) -> Result<(), SetCheckedError> {
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
                SetCheckedError::Internal(InternalError::Failed)
            })?
            .ok_or(SetCheckedError::ItemNotFound)?;

        if item.archived_at().is_some() {
            return Err(SetCheckedError::ItemArchived);
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
                SetCheckedError::Internal(InternalError::Failed)
            })?
            .unwrap_or_else(|| InventoryShoppingState::new(command.household_id, command.item_id));

        if command.checked {
            state.check();
        } else {
            state.uncheck();
        }

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
                SetCheckedError::Internal(InternalError::Failed)
            })?;

        self.household_events_publisher
            .publish(command.household_id, HouseholdEvent::ShoppingListChanged)
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    item_id = %command.item_id,
                    "Failed to publish shopping list changed event"
                );
                SetCheckedError::Internal(InternalError::Failed)
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SetCheckedError {
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
