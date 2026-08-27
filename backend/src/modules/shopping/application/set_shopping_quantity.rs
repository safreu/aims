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

pub struct SetShoppingQuantityCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub item_id: InventoryItemId,
    pub quantity: u32,
}

pub struct SetShoppingQuantityService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    inventory_item_repository: Arc<dyn InventoryItemRepository>,
    shopping_state_repository: Arc<dyn InventoryShoppingStateRepository>,
    household_events_publisher: Arc<dyn HouseholdEventPublisher>,
}

impl SetShoppingQuantityService {
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

    pub async fn execute(
        &self,
        command: SetShoppingQuantityCommand,
    ) -> Result<(), SetShoppingQuantityError> {
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
                SetShoppingQuantityError::Internal(InternalError::Failed)
            })?
            .ok_or(SetShoppingQuantityError::ItemNotFound)?;

        if item.archived_at().is_some() {
            return Err(SetShoppingQuantityError::ItemArchived);
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
                SetShoppingQuantityError::Internal(InternalError::Failed)
            })?
            .unwrap_or_else(|| InventoryShoppingState::new(command.household_id, command.item_id));

        state
            .set_quantity_override(command.quantity)
            .map_err(|_| SetShoppingQuantityError::InvalidQuantity)?;

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
                SetShoppingQuantityError::Internal(InternalError::Failed)
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
                SetShoppingQuantityError::Internal(InternalError::Failed)
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SetShoppingQuantityError {
    #[error("Inventory item not found")]
    ItemNotFound,
    #[error("Inventory item is archived")]
    ItemArchived,
    #[error("Shopping quantity must be greater than zero")]
    InvalidQuantity,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

//TODO: Write these tests
/*
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn quantity_override_can_be_set()

    #[tokio::test]
    async fn state_is_created_when_none_exists()

    #[tokio::test]
    async fn existing_state_is_updated()

    #[tokio::test]
    async fn zero_quantity_is_rejected()

    #[tokio::test]
    async fn missing_item_returns_item_not_found()

    #[tokio::test]
    async fn archived_item_cannot_be_added()

    #[tokio::test]
    async fn non_member_cannot_set_quantity()

    #[tokio::test]
    async fn repository_failure_returns_internal_error()
}
*/
