use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::{HouseholdEvent, HouseholdId},
            ports::{HouseholdAccessError, HouseholdAccessPolicy, HouseholdEventPublisher},
        },
        inventory::{
            domain::{InventoryItemId, InventoryStockEventSource},
            ports::{
                InventoryStockRepository, InventoryStockRepositoryError, StockMutationContext,
            },
        },
    },
    shared::application::InternalError,
};

pub struct DecreaseInventoryStockCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub item_id: InventoryItemId,
    pub amount: u32,
}

pub struct DecreaseInventoryStockService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    inventory_stock_repository: Arc<dyn InventoryStockRepository>,
    household_events_publisher: Arc<dyn HouseholdEventPublisher>,
}

impl DecreaseInventoryStockService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        inventory_stock_repository: Arc<dyn InventoryStockRepository>,
        household_events_publisher: Arc<dyn HouseholdEventPublisher>,
    ) -> Self {
        Self {
            household_access_policy,
            inventory_stock_repository,
            household_events_publisher,
        }
    }

    pub async fn execute(
        &self,
        command: DecreaseInventoryStockCommand,
    ) -> Result<(), DecreaseInventoryStockError> {
        if command.amount == 0 {
            return Err(DecreaseInventoryStockError::InvalidAmount);
        }

        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        //TODO: When implemented replace this with actual source and device_id
        let context = StockMutationContext {
            actor_user_id: Some(command.requester_id),
            actor_device_id: None,
            source: InventoryStockEventSource::Manual,
        };

        self.inventory_stock_repository
            .decrease(
                &command.household_id,
                &command.item_id,
                command.amount,
                &context,
                Utc::now(),
            )
            .await
            .map_err(|error| match error {
                InventoryStockRepositoryError::ItemArchived => {
                    DecreaseInventoryStockError::ItemArchived
                }
                InventoryStockRepositoryError::ItemNotFound => {
                    DecreaseInventoryStockError::ItemNotFound
                }
                InventoryStockRepositoryError::InsufficientStock => {
                    DecreaseInventoryStockError::InsufficientStock
                }
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        item_id = %command.item_id,
                        amount = command.amount,
                        "Failed to decrease inventory stock",
                    );
                    DecreaseInventoryStockError::Internal(InternalError::Failed)
                }
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
                DecreaseInventoryStockError::Internal(InternalError::Failed)
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DecreaseInventoryStockError {
    #[error("Inventory item was not found")]
    ItemNotFound,
    #[error("Inventory item is archived")]
    ItemArchived,
    #[error("Decrease amount is invalid")]
    InvalidAmount,
    #[error("Insufficient stock")]
    InsufficientStock,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

//TODO: Implement the in memory representation of the inventory_stock_repository and write the following tests
/*
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn member_can_decrease_inventory_stock() {}

    #[tokio::test]
    async fn zero_amount_is_rejected() {}

    #[tokio::test]
    async fn archived_item_cannot_be_decreased() {}

    #[tokio::test]
    async fn insufficient_stock_is_rejected() {}

    #[tokio::test]
    async fn unknown_item_returns_not_found() {}

    #[tokio::test]
    async fn non_member_is_forbidden() {}
}
*/
