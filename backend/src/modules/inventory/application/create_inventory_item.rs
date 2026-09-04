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
            domain::{
                CategoryId, InventoryItem, InventoryItemId, InventoryItemName, InventoryPriority,
            },
            ports::{CategoryRepository, InventoryItemRepository, InventoryItemRepositoryError},
        },
    },
    shared::application::InternalError,
};

pub struct CreateInventoryItemCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub category_id: Option<CategoryId>,
    pub name: String,
    pub current_stock: u32,
    pub reorder_threshold: u32,
    pub priority: Option<InventoryPriority>,
}

pub struct CreateInventoryItemService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    category_repository: Arc<dyn CategoryRepository>,
    inventory_item_repository: Arc<dyn InventoryItemRepository>,
    household_events_publisher: Arc<dyn HouseholdEventPublisher>,
}

impl CreateInventoryItemService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        category_repository: Arc<dyn CategoryRepository>,
        inventory_item_repository: Arc<dyn InventoryItemRepository>,
        household_events_publisher: Arc<dyn HouseholdEventPublisher>,
    ) -> Self {
        Self {
            household_access_policy,
            category_repository,
            inventory_item_repository,
            household_events_publisher,
        }
    }

    pub async fn execute(
        &self,
        command: CreateInventoryItemCommand,
    ) -> Result<InventoryItemId, CreateInventoryItemError> {
        let name = InventoryItemName::parse(&command.name)
            .map_err(|_| CreateInventoryItemError::InvalidName)?;

        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        if let Some(category_id) = command.category_id {
            self.category_repository
                .find_by_id(&category_id, &command.household_id)
                .await
                .map_err(|error| {
                    tracing::error!(
                        error = ?error,
                        household_id = %command.household_id,
                        category_id = %category_id,
                        "Failed to load category",
                    );
                    InternalError::Failed
                })?
                .ok_or(CreateInventoryItemError::CategoryNotFound)?;
        }

        let existing = self
            .inventory_item_repository
            .find_active_by_name(&command.household_id, &name)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    item_name = %command.name,
                    "Failed to check inventory item name"
                );
                InternalError::Failed
            })?;

        if existing.is_some() {
            return Err(CreateInventoryItemError::ItemAlreadyExists);
        }

        let now = Utc::now();
        let item_id = InventoryItemId::new();

        let item = InventoryItem::new(
            item_id,
            command.household_id,
            command.category_id,
            name,
            command.current_stock,
            command.reorder_threshold,
            command.priority.unwrap_or_default(),
            now,
            now,
        );

        self.inventory_item_repository
            .insert(&item)
            .await
            .map_err(|error| match error {
                InventoryItemRepositoryError::ItemAlreadyExists => {
                    CreateInventoryItemError::ItemAlreadyExists
                }
                InventoryItemRepositoryError::CategoryNotFound => {
                    CreateInventoryItemError::CategoryNotFound
                }
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        item_id = %item.id(),
                        "Failed to insert inventory item"
                    );
                    CreateInventoryItemError::Internal(InternalError::Failed)
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
                CreateInventoryItemError::Internal(InternalError::Failed)
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
                CreateInventoryItemError::Internal(InternalError::Failed)
            })?;

        Ok(item_id)
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CreateInventoryItemError {
    #[error("Inventory item name is invalid")]
    InvalidName,
    #[error("Category was not found")]
    CategoryNotFound,
    #[error("An inventory item with this name already exists")]
    ItemAlreadyExists,
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
            CategoryTestBuilder, build_create_inventory_item_service, insert_owned_household,
        },
    };

    use super::*;

    #[tokio::test]
    async fn household_member_can_create_inventory_item() {
        let (service, inventory_item_repository, _, household_repository) =
            build_create_inventory_item_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let item_id = service
            .execute(CreateInventoryItemCommand {
                requester_id: owner_id,
                household_id: household.id(),
                category_id: None,
                name: "Tofu".to_owned(),
                current_stock: 3,
                reorder_threshold: 2,
                priority: Some(InventoryPriority::High),
            })
            .await
            .expect("Inventory item creation should succeed");

        let stored = inventory_item_repository
            .find_by_id(&item_id, &household.id())
            .await
            .expect("Inventory item lookup should succeed")
            .expect("Inventory item should exist");

        assert_eq!(stored.household_id(), household.id());
        assert_eq!(stored.category_id(), None);
        assert_eq!(stored.name().normalized(), "tofu");
        assert_eq!(stored.current_stock(), 3);
        assert_eq!(stored.reorder_threshold(), 2);
        assert_eq!(stored.priority(), InventoryPriority::High);
        assert!(stored.archived_at().is_none())
    }

    #[tokio::test]
    async fn created_item_uses_default_priority_when_none_is_provided() {
        let (service, inventory_item_repository, _, household_repository) =
            build_create_inventory_item_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let item_id = service
            .execute(CreateInventoryItemCommand {
                requester_id: owner_id,
                household_id: household.id(),
                category_id: None,
                name: "Tofu".to_owned(),
                current_stock: 3,
                reorder_threshold: 2,
                priority: None,
            })
            .await
            .expect("Inventory item creation should succeed");

        let stored = inventory_item_repository
            .find_by_id(&item_id, &household.id())
            .await
            .expect("Inventory item lookup should succeed")
            .expect("Inventory item should exist");

        assert_eq!(stored.priority(), InventoryPriority::Default);
    }

    #[tokio::test]
    async fn inventory_item_can_be_created_with_category() {
        let (service, inventory_item_repository, category_repository, household_repository) =
            build_create_inventory_item_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let category = CategoryTestBuilder::new(household.id())
            .name("Food".to_owned())
            .build();

        category_repository
            .insert(&category)
            .await
            .expect("Category insertion should succeed");

        let item_id = service
            .execute(CreateInventoryItemCommand {
                requester_id: owner_id,
                household_id: household.id(),
                category_id: Some(category.id()),
                name: "Tofu".to_owned(),
                current_stock: 3,
                reorder_threshold: 2,
                priority: None,
            })
            .await
            .expect("Inventory item creation should succeed");

        let stored = inventory_item_repository
            .find_by_id(&item_id, &household.id())
            .await
            .expect("Inventory item lookup should succeed")
            .expect("Inventory item should exist");

        assert_eq!(stored.category_id(), Some(category.id()));
    }

    #[tokio::test]
    async fn invalid_name_is_rejected() {
        let (service, _, _, household_repository) = build_create_inventory_item_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let result = service
            .execute(CreateInventoryItemCommand {
                requester_id: owner_id,
                household_id: household.id(),
                category_id: None,
                name: "     ".to_owned(),
                current_stock: 3,
                reorder_threshold: 2,
                priority: None,
            })
            .await;

        assert_eq!(result, Err(CreateInventoryItemError::InvalidName))
    }

    #[tokio::test]
    async fn unknown_household_returns_household_not_found() {
        let (service, _, _, household_repository) = build_create_inventory_item_service();

        let owner_id = UserId::new();

        let (_, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let result = service
            .execute(CreateInventoryItemCommand {
                requester_id: owner_id,
                household_id: HouseholdId::new(),
                category_id: None,
                name: "Tofu".to_owned(),
                current_stock: 3,
                reorder_threshold: 2,
                priority: None,
            })
            .await;

        assert_eq!(
            result,
            Err(CreateInventoryItemError::HouseholdAccess(
                HouseholdAccessError::HouseholdNotFound
            ))
        )
    }

    #[tokio::test]
    async fn non_member_returns_forbidden() {
        let (service, _, _, household_repository) = build_create_inventory_item_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let result = service
            .execute(CreateInventoryItemCommand {
                requester_id: UserId::new(),
                household_id: household.id(),
                category_id: None,
                name: "Tofu".to_owned(),
                current_stock: 3,
                reorder_threshold: 2,
                priority: None,
            })
            .await;

        assert_eq!(
            result,
            Err(CreateInventoryItemError::HouseholdAccess(
                HouseholdAccessError::Forbidden
            ))
        )
    }

    #[tokio::test]
    async fn category_from_different_household_returns_category_not_found() {
        let (service, _, category_repository, household_repository) =
            build_create_inventory_item_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let (another_household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let category = CategoryTestBuilder::new(household.id())
            .name("Food".to_owned())
            .build();

        category_repository
            .insert(&category)
            .await
            .expect("Category insertion should succeed");

        let result = service
            .execute(CreateInventoryItemCommand {
                requester_id: owner_id,
                household_id: another_household.id(),
                category_id: Some(category.id()),
                name: "Tofu".to_owned(),
                current_stock: 3,
                reorder_threshold: 2,
                priority: None,
            })
            .await;

        assert_eq!(result, Err(CreateInventoryItemError::CategoryNotFound))
    }

    #[tokio::test]
    async fn duplicate_active_item_returns_item_already_exists() {
        let (service, _, _, household_repository) = build_create_inventory_item_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        service
            .execute(CreateInventoryItemCommand {
                requester_id: owner_id,
                household_id: household.id(),
                category_id: None,
                name: "Tofu".to_owned(),
                current_stock: 3,
                reorder_threshold: 2,
                priority: None,
            })
            .await
            .expect("Inventory item creation should succeed");

        let result = service
            .execute(CreateInventoryItemCommand {
                requester_id: owner_id,
                household_id: household.id(),
                category_id: None,
                name: "tofu".to_owned(),
                current_stock: 5,
                reorder_threshold: 1,
                priority: None,
            })
            .await;

        assert_eq!(result, Err(CreateInventoryItemError::ItemAlreadyExists))
    }
}
