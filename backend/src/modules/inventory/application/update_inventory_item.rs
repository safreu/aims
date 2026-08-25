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
            domain::{
                CategoryId, InventoryItemError, InventoryItemId, InventoryItemName,
                InventoryPriority,
            },
            ports::{CategoryRepository, InventoryItemRepository, InventoryItemRepositoryError},
        },
    },
    shared::application::InternalError,
};

pub struct UpdateInventoryItemCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub item_id: InventoryItemId,
    pub name: Option<String>,
    pub category_id: Option<Option<CategoryId>>,
    pub reorder_threshold: Option<u32>,
    pub priority: Option<String>,
}

pub struct UpdateInventoryItemService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    category_repository: Arc<dyn CategoryRepository>,
    inventory_item_repository: Arc<dyn InventoryItemRepository>,
}

impl UpdateInventoryItemService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        category_repository: Arc<dyn CategoryRepository>,
        inventory_item_repository: Arc<dyn InventoryItemRepository>,
    ) -> Self {
        Self {
            household_access_policy,
            category_repository,
            inventory_item_repository,
        }
    }

    pub async fn execute(
        &self,
        command: UpdateInventoryItemCommand,
    ) -> Result<(), UpdateInventoryItemError> {
        if command.category_id.is_none()
            && command.name.is_none()
            && command.reorder_threshold.is_none()
            && command.priority.is_none()
        {
            return Err(UpdateInventoryItemError::NoChanges);
        }

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
                    "Failed to load inventory item"
                );
                InternalError::Failed
            })?
            .ok_or(UpdateInventoryItemError::ItemNotFound)?;

        let name = command
            .name
            .map(|name| {
                InventoryItemName::parse(&name).map_err(|_| UpdateInventoryItemError::InvalidName)
            })
            .transpose()?;

        let priority = command
            .priority
            .map(|priority| {
                InventoryPriority::parse(&priority)
                    .map_err(|_| UpdateInventoryItemError::InvalidPriority)
            })
            .transpose()?;

        if let Some(Some(category_id)) = command.category_id {
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
                .ok_or(UpdateInventoryItemError::CategoryNotFound)?;
        }

        let now = Utc::now();

        if let Some(name) = name {
            item.rename(name, now).map_err(map_inventory_item_error)?;
        }

        if let Some(category_id) = command.category_id {
            item.change_category(category_id, now)
                .map_err(map_inventory_item_error)?;
        }

        if let Some(reorder_threshold) = command.reorder_threshold {
            item.set_reorder_threshold(reorder_threshold, now)
                .map_err(map_inventory_item_error)?;
        }

        if let Some(priority) = priority {
            item.set_priority(priority, now)
                .map_err(map_inventory_item_error)?;
        }

        self.inventory_item_repository
            .update(&item)
            .await
            .map_err(|error| match error {
                InventoryItemRepositoryError::ItemNotFound => {
                    UpdateInventoryItemError::ItemNotFound
                }
                InventoryItemRepositoryError::ItemAlreadyExists => {
                    UpdateInventoryItemError::ItemAlreadyExists
                }
                InventoryItemRepositoryError::CategoryNotFound => {
                    UpdateInventoryItemError::CategoryNotFound
                }
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        item_id = %command.item_id,
                        "Failed to update inventory item"
                    );
                    UpdateInventoryItemError::Internal(InternalError::Failed)
                }
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum UpdateInventoryItemError {
    #[error("No changes were provided")]
    NoChanges,
    #[error("Inventory item name is invalid")]
    InvalidName,
    #[error("Inventory item priority is invalid")]
    InvalidPriority,
    #[error("Inventory item was not found")]
    ItemNotFound,
    #[error("Category was not found")]
    CategoryNotFound,
    #[error("An active inventory item with this name already exists")]
    ItemAlreadyExists,
    #[error("Archived inventory items cannot be modified")]
    ItemArchived,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

fn map_inventory_item_error(error: InventoryItemError) -> UpdateInventoryItemError {
    match error {
        InventoryItemError::Archived => UpdateInventoryItemError::ItemArchived,
        other => {
            tracing::error!(
                error = ?other,
                "Unexpected inventory item error while updating metadata"
            );
            UpdateInventoryItemError::Internal(InternalError::Failed)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        modules::households::domain::HouseholdKind,
        test_helpers::{
            CategoryTestBuilder, InventoryItemTestBuilder, build_update_inventory_item_service,
            insert_owned_household,
        },
    };

    use super::*;

    #[tokio::test]
    async fn member_can_update_inventory_item_metadata() {
        let (service, category_repository, inventory_repository, household_repository) =
            build_update_inventory_item_service();

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

        let item = InventoryItemTestBuilder::new(household.id())
            .name("Tofu".to_owned())
            .build();

        inventory_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        let command = UpdateInventoryItemCommand {
            requester_id: owner_id,
            household_id: household.id(),
            item_id: item.id(),
            name: Some("Apple".to_owned()),
            category_id: Some(Some(category.id())),
            reorder_threshold: Some(1),
            priority: Some("low".to_owned()),
        };

        service
            .execute(command)
            .await
            .expect("Inventory item update should succeed");

        let stored = inventory_repository
            .find_by_id(&item.id(), &household.id())
            .await
            .expect("Inventory item lookup should succeed")
            .expect("Inventory item should exist");

        assert_eq!(stored.name().as_str(), "Apple");
        assert_eq!(stored.category_id(), Some(category.id()));
        assert_eq!(stored.reorder_threshold(), 1);
        assert_eq!(stored.priority(), InventoryPriority::Low);
    }

    #[tokio::test]
    async fn category_can_be_removed() {
        let (service, category_repository, inventory_repository, household_repository) =
            build_update_inventory_item_service();

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

        let item = InventoryItemTestBuilder::new(household.id())
            .name("Tofu".to_owned())
            .build();

        inventory_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        let command = UpdateInventoryItemCommand {
            requester_id: owner_id,
            household_id: household.id(),
            item_id: item.id(),
            name: Some("Apple".to_owned()),
            category_id: Some(None),
            reorder_threshold: Some(1),
            priority: Some("low".to_owned()),
        };

        service
            .execute(command)
            .await
            .expect("Inventory item update should succeed");

        let stored = inventory_repository
            .find_by_id(&item.id(), &household.id())
            .await
            .expect("Inventory item lookup should succeed")
            .expect("Inventory item should exist");

        assert_eq!(stored.category_id(), None);
    }

    #[tokio::test]
    async fn empty_update_is_rejected() {
        let (service, category_repository, inventory_repository, household_repository) =
            build_update_inventory_item_service();

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

        let item = InventoryItemTestBuilder::new(household.id())
            .name("Tofu".to_owned())
            .build();

        inventory_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        let command = UpdateInventoryItemCommand {
            requester_id: owner_id,
            household_id: household.id(),
            item_id: item.id(),
            name: None,
            category_id: None,
            reorder_threshold: None,
            priority: None,
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(UpdateInventoryItemError::NoChanges));
    }

    #[tokio::test]
    async fn invalid_name_is_rejected() {
        let (service, category_repository, inventory_repository, household_repository) =
            build_update_inventory_item_service();

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

        let item = InventoryItemTestBuilder::new(household.id())
            .name("Tofu".to_owned())
            .build();

        inventory_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        let command = UpdateInventoryItemCommand {
            requester_id: owner_id,
            household_id: household.id(),
            item_id: item.id(),
            name: Some("    ".to_owned()),
            category_id: None,
            reorder_threshold: None,
            priority: None,
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(UpdateInventoryItemError::InvalidName));
    }

    #[tokio::test]
    async fn invalid_priority_is_rejected() {
        let (service, category_repository, inventory_repository, household_repository) =
            build_update_inventory_item_service();

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

        let item = InventoryItemTestBuilder::new(household.id())
            .name("Tofu".to_owned())
            .build();

        inventory_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        let command = UpdateInventoryItemCommand {
            requester_id: owner_id,
            household_id: household.id(),
            item_id: item.id(),
            name: None,
            category_id: None,
            reorder_threshold: None,
            priority: Some("invalid".to_owned()),
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(UpdateInventoryItemError::InvalidPriority));
    }

    #[tokio::test]
    async fn category_from_different_household_is_rejected() {
        let (service, category_repository, inventory_repository, household_repository) =
            build_update_inventory_item_service();

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

        let item = InventoryItemTestBuilder::new(household.id())
            .name("Tofu".to_owned())
            .build();

        inventory_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        let command = UpdateInventoryItemCommand {
            requester_id: owner_id,
            household_id: HouseholdId::new(),
            item_id: item.id(),
            name: None,
            category_id: None,
            reorder_threshold: None,
            priority: Some("low".to_owned()),
        };

        let result = service.execute(command).await;

        assert_eq!(
            result,
            Err(UpdateInventoryItemError::HouseholdAccess(
                HouseholdAccessError::HouseholdNotFound
            ))
        );
    }

    #[tokio::test]
    async fn archived_item_cannot_be_updated() {
        let (service, category_repository, inventory_repository, household_repository) =
            build_update_inventory_item_service();

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

        let mut item = InventoryItemTestBuilder::new(household.id())
            .name("Tofu".to_owned())
            .build();

        inventory_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        item.archive(Utc::now())
            .expect("Inventory item archiving should succeed");

        inventory_repository
            .update(&item)
            .await
            .expect("Inventory item updating should succeed");

        let command = UpdateInventoryItemCommand {
            requester_id: owner_id,
            household_id: household.id(),
            item_id: item.id(),
            name: None,
            category_id: None,
            reorder_threshold: None,
            priority: Some("low".to_owned()),
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(UpdateInventoryItemError::ItemArchived));
    }

    #[tokio::test]
    async fn duplicate_active_name_is_rejected() {
        let (service, category_repository, inventory_repository, household_repository) =
            build_update_inventory_item_service();

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

        let item = InventoryItemTestBuilder::new(household.id())
            .name("Tofu".to_owned())
            .build();

        inventory_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        let another_item = InventoryItemTestBuilder::new(household.id())
            .name("Apple".to_owned())
            .build();

        inventory_repository
            .insert(&another_item)
            .await
            .expect("Inventory item insertion should succeed");

        let command = UpdateInventoryItemCommand {
            requester_id: owner_id,
            household_id: household.id(),
            item_id: item.id(),
            name: Some("apple".to_owned()),
            category_id: None,
            reorder_threshold: None,
            priority: Some("low".to_owned()),
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(UpdateInventoryItemError::ItemAlreadyExists));
    }

    #[tokio::test]
    async fn non_member_is_forbidden() {
        let (service, category_repository, inventory_repository, household_repository) =
            build_update_inventory_item_service();

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

        let item = InventoryItemTestBuilder::new(household.id())
            .name("Tofu".to_owned())
            .build();

        inventory_repository
            .insert(&item)
            .await
            .expect("Inventory item insertion should succeed");

        let command = UpdateInventoryItemCommand {
            requester_id: UserId::new(),
            household_id: household.id(),
            item_id: item.id(),
            name: Some("apple".to_owned()),
            category_id: None,
            reorder_threshold: None,
            priority: Some("low".to_owned()),
        };

        let result = service.execute(command).await;

        assert_eq!(
            result,
            Err(UpdateInventoryItemError::HouseholdAccess(
                HouseholdAccessError::Forbidden
            ))
        );
    }
}
