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
            domain::{InventoryItemError, InventoryItemId},
            ports::{InventoryItemRepository, InventoryItemRepositoryError},
        },
    },
    shared::application::InternalError,
};

pub struct RestoreInventoryItemCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub item_id: InventoryItemId,
}

pub struct RestoreInventoryItemService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    inventory_item_repository: Arc<dyn InventoryItemRepository>,
    household_events_publisher: Arc<dyn HouseholdEventPublisher>,
}

impl RestoreInventoryItemService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        inventory_item_repository: Arc<dyn InventoryItemRepository>,
        household_events_publisher: Arc<dyn HouseholdEventPublisher>,
    ) -> Self {
        Self {
            household_access_policy,
            inventory_item_repository,
            household_events_publisher,
        }
    }

    pub async fn execute(
        &self,
        command: RestoreInventoryItemCommand,
    ) -> Result<(), RestoreInventoryItemError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let mut item = self
            .inventory_item_repository
            .find_by_id(&command.item_id, &command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    item_id = %command.item_id,
                    "Failed to load inventory item",
                );
                InternalError::Failed
            })?
            .ok_or(RestoreInventoryItemError::ItemNotFound)?;

        item.restore(Utc::now()).map_err(|error| match error {
            InventoryItemError::NotArchived => RestoreInventoryItemError::NotArchived,
            other => {
                tracing::error!(
                    error = ?other,
                    "Unexpected inventory item error while restoring archived item",
                );
                RestoreInventoryItemError::Internal(InternalError::Failed)
            }
        })?;

        self.inventory_item_repository
            .update(&item)
            .await
            .map_err(|error| match error {
                InventoryItemRepositoryError::ItemNotFound => {
                    RestoreInventoryItemError::ItemNotFound
                }
                InventoryItemRepositoryError::ItemAlreadyExists => {
                    RestoreInventoryItemError::ItemAlreadyExists
                }
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        item_id = %item.id(),
                        "Failed to persist restored inventory item"
                    );
                    RestoreInventoryItemError::Internal(InternalError::Failed)
                }
            })?;

        self.household_events_publisher
            .publish(command.household_id, HouseholdEvent::ShoppingListChanged)
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    item_id = %item.id(),
                    "Failed to publish shopping list changed event"
                );
                RestoreInventoryItemError::Internal(InternalError::Failed)
            })?;

        self.household_events_publisher
            .publish(command.household_id, HouseholdEvent::InventoryItemsChanged)
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    item_id = %item.id(),
                    "Failed to publish inventory items changed event"
                );
                RestoreInventoryItemError::Internal(InternalError::Failed)
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RestoreInventoryItemError {
    #[error("Active inventory item with this name already exists")]
    ItemAlreadyExists,
    #[error("Inventory item was not found")]
    ItemNotFound,
    #[error("Inventory item is not archived")]
    NotArchived,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

#[cfg(test)]
mod tests {
    use crate::{
        modules::households::domain::HouseholdKind,
        test_helpers::{
            InventoryItemTestBuilder, build_restore_item_service, insert_owned_household,
        },
    };

    use super::*;

    #[tokio::test]
    async fn member_can_restore_inventory_item() {
        let (service, inventory_item_repository, household_repository) =
            build_restore_item_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let mut item = InventoryItemTestBuilder::new(household.id())
            .name("Tofu".to_owned())
            .build();

        inventory_item_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        item.archive(Utc::now())
            .expect("Item archiving should succeed");

        inventory_item_repository
            .update(&item)
            .await
            .expect("Inventory item update should succeed");

        let command = RestoreInventoryItemCommand {
            requester_id: owner_id,
            household_id: household.id(),
            item_id: item.id(),
        };

        let result = service.execute(command).await;

        assert!(result.is_ok());

        let stored = inventory_item_repository
            .find_by_id(&item.id(), &household.id())
            .await
            .expect("Inventory item lookup should succeed")
            .expect("Item should exists");

        assert!(stored.archived_at().is_none());
    }

    #[tokio::test]
    async fn restoring_unknown_item_returns_not_found() {
        let (service, _, household_repository) = build_restore_item_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let item = InventoryItemTestBuilder::new(household.id())
            .name("Tofu".to_owned())
            .build();

        let command = RestoreInventoryItemCommand {
            requester_id: owner_id,
            household_id: household.id(),
            item_id: item.id(),
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(RestoreInventoryItemError::ItemNotFound))
    }

    #[tokio::test]
    async fn restoring_active_item_is_rejected() {
        let (service, inventory_item_repository, household_repository) =
            build_restore_item_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let item = InventoryItemTestBuilder::new(household.id())
            .name("Tofu".to_owned())
            .build();

        inventory_item_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        let command = RestoreInventoryItemCommand {
            requester_id: owner_id,
            household_id: household.id(),
            item_id: item.id(),
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(RestoreInventoryItemError::NotArchived))
    }

    #[tokio::test]
    async fn non_member_is_forbidden() {
        let (service, inventory_item_repository, household_repository) =
            build_restore_item_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let item = InventoryItemTestBuilder::new(household.id())
            .name("Tofu".to_owned())
            .build();

        inventory_item_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        let command = RestoreInventoryItemCommand {
            requester_id: UserId::new(),
            household_id: household.id(),
            item_id: item.id(),
        };

        let result = service.execute(command).await;

        assert_eq!(
            result,
            Err(RestoreInventoryItemError::HouseholdAccess(
                HouseholdAccessError::Forbidden
            ))
        )
    }

    #[tokio::test]
    async fn restoring_duplicate_active_name_is_rejected() {
        let (service, inventory_item_repository, household_repository) =
            build_restore_item_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let mut item = InventoryItemTestBuilder::new(household.id())
            .name("Tofu".to_owned())
            .build();

        inventory_item_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        item.archive(Utc::now())
            .expect("Item archiving should succeed");

        inventory_item_repository
            .update(&item)
            .await
            .expect("Inventory item update should succeed");

        let another_item = InventoryItemTestBuilder::new(household.id())
            .name("tofu".to_owned())
            .build();

        inventory_item_repository
            .insert(&another_item)
            .await
            .expect("Inventory item insertion should succeed");

        let command = RestoreInventoryItemCommand {
            requester_id: owner_id,
            household_id: household.id(),
            item_id: item.id(),
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(RestoreInventoryItemError::ItemAlreadyExists))
    }
}
