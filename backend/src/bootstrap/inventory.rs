use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    modules::{
        households::adapters::{DefaultHouseholdAccessPolicy, PostgresHouseholdRepository},
        inventory::{
            adapters::{
                PostgresCategoryRepository, PostgresInventoryItemQuery,
                PostgresInventoryItemRepository, PostgresInventoryStockHistoryQuery,
                PostgresInventoryStockRepository,
            },
            application::{
                ArchiveInventoryItemService, CreateCategoryService, CreateInventoryItemService,
                DecreaseInventoryStockService, DeleteCategoryService, GetInventoryItemService,
                IncreaseInventoryStockService, ListCategoriesService, ListInventoryItemsService,
                ListInventoryStockHistoryService, RestoreInventoryItemService,
                SetInventoryStockService, UpdateInventoryItemService,
            },
        },
    },
    shared::api::InventoryItemState,
};

pub(super) fn build_inventory_item_state(pool: &PgPool) -> InventoryItemState {
    let household_repository = Arc::new(PostgresHouseholdRepository::new(pool.clone()));
    let household_access_policy = Arc::new(DefaultHouseholdAccessPolicy::new(household_repository));
    let category_repository = Arc::new(PostgresCategoryRepository::new(pool.clone()));
    let inventory_item_repository = Arc::new(PostgresInventoryItemRepository::new(pool.clone()));
    let inventory_item_query = Arc::new(PostgresInventoryItemQuery::new(pool.clone()));
    let inventory_stock_repository = Arc::new(PostgresInventoryStockRepository::new(pool.clone()));
    let inventory_stock_history_query =
        Arc::new(PostgresInventoryStockHistoryQuery::new(pool.clone()));

    let create_inventory_item_service = Arc::new(CreateInventoryItemService::new(
        household_access_policy.clone(),
        category_repository.clone(),
        inventory_item_repository.clone(),
    ));

    let create_category_service = Arc::new(CreateCategoryService::new(
        household_access_policy.clone(),
        category_repository.clone(),
    ));

    let list_categories_service = Arc::new(ListCategoriesService::new(
        household_access_policy.clone(),
        category_repository.clone(),
    ));

    let delete_category_service = Arc::new(DeleteCategoryService::new(
        household_access_policy.clone(),
        category_repository.clone(),
    ));

    let list_inventory_items_service = Arc::new(ListInventoryItemsService::new(
        household_access_policy.clone(),
        inventory_item_query.clone(),
    ));

    let get_inventory_item_service = Arc::new(GetInventoryItemService::new(
        household_access_policy.clone(),
        inventory_item_query.clone(),
    ));

    let update_inventory_item_service = Arc::new(UpdateInventoryItemService::new(
        household_access_policy.clone(),
        category_repository,
        inventory_item_repository.clone(),
    ));

    let archive_inventory_item_service = Arc::new(ArchiveInventoryItemService::new(
        household_access_policy.clone(),
        inventory_item_repository.clone(),
    ));

    let restore_inventory_item_service = Arc::new(RestoreInventoryItemService::new(
        household_access_policy.clone(),
        inventory_item_repository.clone(),
    ));

    let increase_inventory_stock_service = Arc::new(IncreaseInventoryStockService::new(
        household_access_policy.clone(),
        inventory_stock_repository.clone(),
    ));

    let decrease_inventory_stock_service = Arc::new(DecreaseInventoryStockService::new(
        household_access_policy.clone(),
        inventory_stock_repository.clone(),
    ));

    let set_inventory_stock_service = Arc::new(SetInventoryStockService::new(
        household_access_policy.clone(),
        inventory_stock_repository.clone(),
    ));

    let list_inventory_stock_history_service = Arc::new(ListInventoryStockHistoryService::new(
        household_access_policy.clone(),
        inventory_item_repository.clone(),
        inventory_stock_history_query.clone(),
    ));

    InventoryItemState {
        create_inventory_item: create_inventory_item_service,
        create_category: create_category_service,
        list_categories: list_categories_service,
        delete_category: delete_category_service,
        list_inventory_items: list_inventory_items_service,
        get_inventory_item: get_inventory_item_service,
        update_inventory_item: update_inventory_item_service,
        archive_inventory_item: archive_inventory_item_service,
        restore_inventory_item: restore_inventory_item_service,
        increase_inventory_stock: increase_inventory_stock_service,
        decrease_inventory_stock: decrease_inventory_stock_service,
        set_inventory_stock: set_inventory_stock_service,
        list_inventory_stock_history: list_inventory_stock_history_service,
    }
}
