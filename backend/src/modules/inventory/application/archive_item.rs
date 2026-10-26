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
            domain::{InventoryEvent, InventoryItemError, InventoryItemId},
            ports::{InventoryItemRepository, InventoryItemRepositoryError},
        },
    },
    shared::application::{InternalError, events::ApplicationResult},
};

pub struct ArchiveInventoryItemCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub item_id: InventoryItemId,
}

pub struct ArchiveInventoryItemService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    inventory_item_repository: Arc<dyn InventoryItemRepository>,
}

impl ArchiveInventoryItemService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        inventory_item_repository: Arc<dyn InventoryItemRepository>,
    ) -> Self {
        Self {
            household_access_policy,
            inventory_item_repository,
        }
    }

    pub async fn execute(
        &self,
        command: ArchiveInventoryItemCommand,
    ) -> Result<ApplicationResult<()>, ArchiveInventoryItemError> {
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
            .ok_or(ArchiveInventoryItemError::ItemNotFound)?;

        item.archive(Utc::now()).map_err(|error| match error {
            InventoryItemError::AlreadyArchived => ArchiveInventoryItemError::AlreadyArchived,
            other => {
                tracing::error!(
                    error = ?other,
                    "Unexpected inventory item error while archiving item",
                );
                ArchiveInventoryItemError::Internal(InternalError::Failed)
            }
        })?;

        self.inventory_item_repository
            .update(&item)
            .await
            .map_err(|error| match error {
                InventoryItemRepositoryError::ItemNotFound => {
                    ArchiveInventoryItemError::ItemNotFound
                }
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        item_id = %item.id(),
                        "Failed to persist archived inventory item"
                    );
                    ArchiveInventoryItemError::Internal(InternalError::Failed)
                }
            })?;

        Ok(
            ApplicationResult::new(()).with_event(InventoryEvent::ItemArchived {
                household_id: command.household_id,
                item_id: command.item_id,
            }),
        )
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ArchiveInventoryItemError {
    #[error("Inventory item was not found")]
    ItemNotFound,
    #[error("Inventory item was already archived")]
    AlreadyArchived,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

#[cfg(test)]
mod tests {
    use crate::{
        modules::{households::domain::HouseholdKind, inventory::domain::InventoryEvent},
        shared::application::events::AppEvent,
        test_helpers::{
            InventoryItemTestBuilder, build_archive_item_service, insert_owned_household,
        },
    };

    use super::*;

    #[tokio::test]
    async fn member_can_archive_inventory_item() {
        let (service, inventory_item_repository, household_repository) =
            build_archive_item_service();

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

        service
            .execute(ArchiveInventoryItemCommand {
                requester_id: owner_id,
                household_id: household.id(),
                item_id: item.id(),
            })
            .await
            .expect("Inventory item archiving should succeed");

        let stored = inventory_item_repository
            .find_by_id(&item.id(), &household.id())
            .await
            .expect("Inventory item lookup should succeed")
            .expect("Inventory item should exist");

        assert!(stored.archived_at().is_some());
    }

    #[tokio::test]
    async fn archiving_inventory_item_emits_item_archived_event() {
        let (service, inventory_item_repository, household_repository) =
            build_archive_item_service();

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

        let result = service
            .execute(ArchiveInventoryItemCommand {
                requester_id: owner_id,
                household_id: household.id(),
                item_id: item.id(),
            })
            .await
            .expect("Inventory item archiving should succeed");

        let (_, events) = result.into_parts();

        assert_eq!(
            events,
            vec![AppEvent::Inventory(InventoryEvent::ItemArchived {
                household_id: household.id(),
                item_id: item.id(),
            })]
        );
    }

    #[tokio::test]
    async fn archiving_unknown_item_returns_not_found() {
        let (service, _, household_repository) = build_archive_item_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let item_id = InventoryItemId::new();

        let result = service
            .execute(ArchiveInventoryItemCommand {
                requester_id: owner_id,
                household_id: household.id(),
                item_id,
            })
            .await;

        assert!(matches!(
            result,
            Err(ArchiveInventoryItemError::ItemNotFound)
        ));
    }

    #[tokio::test]
    async fn archiving_already_archived_item_is_rejected() {
        let (service, inventory_item_repository, household_repository) =
            build_archive_item_service();

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

        service
            .execute(ArchiveInventoryItemCommand {
                requester_id: owner_id,
                household_id: household.id(),
                item_id: item.id(),
            })
            .await
            .expect("Inventory item archiving should succeed");

        let result = service
            .execute(ArchiveInventoryItemCommand {
                requester_id: owner_id,
                household_id: household.id(),
                item_id: item.id(),
            })
            .await;

        assert!(matches!(
            result,
            Err(ArchiveInventoryItemError::AlreadyArchived)
        ));
    }

    #[tokio::test]
    async fn non_member_is_forbidden() {
        let (service, inventory_item_repository, household_repository) =
            build_archive_item_service();

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

        let result = service
            .execute(ArchiveInventoryItemCommand {
                requester_id: UserId::new(),
                household_id: household.id(),
                item_id: item.id(),
            })
            .await;

        assert!(matches!(
            result,
            Err(ArchiveInventoryItemError::HouseholdAccess(
                HouseholdAccessError::Forbidden
            ))
        ));
    }
}
