use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
        inventory::{
            domain::{InventoryEvent, InventoryItemId, InventoryStockEventSource},
            ports::{
                InventoryStockRepository, InventoryStockRepositoryError, StockMutationContext,
            },
        },
    },
    shared::application::{InternalError, events::ApplicationResult},
};

pub struct SetInventoryStockCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub item_id: InventoryItemId,
    pub stock: u32,
}

pub struct SetInventoryStockService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    inventory_stock_repository: Arc<dyn InventoryStockRepository>,
}

impl SetInventoryStockService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        inventory_stock_repository: Arc<dyn InventoryStockRepository>,
    ) -> Self {
        Self {
            household_access_policy,
            inventory_stock_repository,
        }
    }

    pub async fn execute(
        &self,
        command: SetInventoryStockCommand,
    ) -> Result<ApplicationResult<()>, SetInventoryStockError> {
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
            .set(
                &command.household_id,
                &command.item_id,
                command.stock,
                &context,
                Utc::now(),
            )
            .await
            .map_err(|error| match error {
                InventoryStockRepositoryError::ItemArchived => SetInventoryStockError::ItemArchived,
                InventoryStockRepositoryError::ItemNotFound => SetInventoryStockError::ItemNotFound,
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        item_id = %command.item_id,
                        amount = command.stock,
                        "Failed to set inventory stock",
                    );
                    SetInventoryStockError::Internal(InternalError::Failed)
                }
            })?;

        Ok(
            ApplicationResult::new(()).with_event(InventoryEvent::StockChanged {
                household_id: command.household_id,
                item_id: command.item_id,
            }),
        )
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SetInventoryStockError {
    #[error("Inventory item was not found")]
    ItemNotFound,
    #[error("Inventory item is archived")]
    ItemArchived,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::{
        modules::{
            households::{domain::HouseholdKind, ports::HouseholdAccessError},
            inventory::{domain::InventoryEvent, ports::InventoryItemRepository},
        },
        shared::application::events::AppEvent,
        test_helpers::{
            InventoryItemTestBuilder, build_set_inventory_stock_service, insert_owned_household,
        },
    };

    use super::*;

    #[tokio::test]
    async fn member_can_set_inventory_stock() {
        let (service, inventory_item_repository, household_repository) =
            build_set_inventory_stock_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let item = InventoryItemTestBuilder::new(household.id())
            .current_stock(3)
            .build();

        inventory_item_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        service
            .execute(SetInventoryStockCommand {
                requester_id: owner_id,
                household_id: household.id(),
                item_id: item.id(),
                stock: 10,
            })
            .await
            .expect("Setting inventory stock should succeed");

        let stored = inventory_item_repository
            .find_by_id(&item.id(), &household.id())
            .await
            .expect("Inventory item lookup should succeed")
            .expect("Inventory item should exist");

        assert_eq!(stored.current_stock(), 10);
    }

    #[tokio::test]
    async fn setting_inventory_stock_emits_stock_changed_event() {
        let (service, inventory_item_repository, household_repository) =
            build_set_inventory_stock_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let item = InventoryItemTestBuilder::new(household.id())
            .current_stock(3)
            .build();

        inventory_item_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        let result = service
            .execute(SetInventoryStockCommand {
                requester_id: owner_id,
                household_id: household.id(),
                item_id: item.id(),
                stock: 10,
            })
            .await
            .expect("Setting inventory stock should succeed");

        let (_, events) = result.into_parts();

        assert_eq!(
            events,
            vec![AppEvent::Inventory(InventoryEvent::StockChanged {
                household_id: household.id(),
                item_id: item.id(),
            })]
        );
    }

    #[tokio::test]
    async fn stock_can_be_set_to_zero() {
        let (service, inventory_item_repository, household_repository) =
            build_set_inventory_stock_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let item = InventoryItemTestBuilder::new(household.id())
            .current_stock(3)
            .build();

        inventory_item_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        service
            .execute(SetInventoryStockCommand {
                requester_id: owner_id,
                household_id: household.id(),
                item_id: item.id(),
                stock: 0,
            })
            .await
            .expect("Setting inventory stock to zero should succeed");

        let stored = inventory_item_repository
            .find_by_id(&item.id(), &household.id())
            .await
            .expect("Inventory item lookup should succeed")
            .expect("Inventory item should exist");

        assert_eq!(stored.current_stock(), 0);
    }

    #[tokio::test]
    async fn archived_item_cannot_have_stock_set() {
        let (service, inventory_item_repository, household_repository) =
            build_set_inventory_stock_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let mut item = InventoryItemTestBuilder::new(household.id())
            .current_stock(3)
            .build();

        item.archive(Utc::now())
            .expect("Inventory item archiving should succeed");

        inventory_item_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        let result = service
            .execute(SetInventoryStockCommand {
                requester_id: owner_id,
                household_id: household.id(),
                item_id: item.id(),
                stock: 10,
            })
            .await;

        assert!(matches!(result, Err(SetInventoryStockError::ItemArchived)));
    }

    #[tokio::test]
    async fn unknown_item_returns_not_found() {
        let (service, _, household_repository) = build_set_inventory_stock_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let result = service
            .execute(SetInventoryStockCommand {
                requester_id: owner_id,
                household_id: household.id(),
                item_id: InventoryItemId::new(),
                stock: 10,
            })
            .await;

        assert!(matches!(result, Err(SetInventoryStockError::ItemNotFound)));
    }

    #[tokio::test]
    async fn non_member_is_forbidden() {
        let (service, inventory_item_repository, household_repository) =
            build_set_inventory_stock_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let item = InventoryItemTestBuilder::new(household.id())
            .current_stock(3)
            .build();

        inventory_item_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        let result = service
            .execute(SetInventoryStockCommand {
                requester_id: UserId::new(),
                household_id: household.id(),
                item_id: item.id(),
                stock: 10,
            })
            .await;

        assert!(matches!(
            result,
            Err(SetInventoryStockError::HouseholdAccess(
                HouseholdAccessError::Forbidden
            ))
        ));
    }
}
