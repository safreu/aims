use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    modules::{
        households::{
            adapters::{DefaultHouseholdAccessPolicy, PostgresHouseholdRepository},
            ports::HouseholdEventPublisher,
        },
        inventory::adapters::{
            PostgresInventoryItemRepository, PostgresInventoryStockHistoryQuery,
        },
        shopping::{
            adapters::{
                PostgresCustomShoppingEntryRepository, PostgresInventoryShoppingStateRepository,
                PostgresShoppingListQuery,
            },
            application::{
                CreateCustomShoppingEntryService, DeleteCustomShoppingEntryService,
                DismissShoppingItemService, ListShoppingService, SetCheckedService,
                SetCustomShoppingEntryCheckedService, SetNoteService, SetShoppingQuantityService,
                UpdateCustomShoppingEntryService,
            },
        },
    },
    shared::api::ShoppingState,
};

pub(super) fn build_shopping_state(
    pool: &PgPool,
    household_events_publisher: Arc<dyn HouseholdEventPublisher>,
) -> ShoppingState {
    let household_repository = Arc::new(PostgresHouseholdRepository::new(pool.clone()));
    let household_access_policy = Arc::new(DefaultHouseholdAccessPolicy::new(household_repository));
    let inventory_item_repository = Arc::new(PostgresInventoryItemRepository::new(pool.clone()));
    Arc::new(PostgresInventoryStockHistoryQuery::new(pool.clone()));
    let shopping_list_query = Arc::new(PostgresShoppingListQuery::new(pool.clone()));
    let custom_entry_repository =
        Arc::new(PostgresCustomShoppingEntryRepository::new(pool.clone()));
    let shopping_state_repository =
        Arc::new(PostgresInventoryShoppingStateRepository::new(pool.clone()));

    let list_shopping_service = Arc::new(ListShoppingService::new(
        household_access_policy.clone(),
        shopping_list_query.clone(),
        custom_entry_repository.clone(),
    ));

    let create_custom_shopping_entry_service = Arc::new(CreateCustomShoppingEntryService::new(
        household_access_policy.clone(),
        custom_entry_repository.clone(),
        household_events_publisher.clone(),
    ));

    let delete_custom_shopping_entry_service = Arc::new(DeleteCustomShoppingEntryService::new(
        household_access_policy.clone(),
        custom_entry_repository.clone(),
        household_events_publisher.clone(),
    ));

    let set_custom_shopping_entry_checked_service =
        Arc::new(SetCustomShoppingEntryCheckedService::new(
            household_access_policy.clone(),
            custom_entry_repository.clone(),
            household_events_publisher.clone(),
        ));

    let update_custom_shopping_entry_service = Arc::new(UpdateCustomShoppingEntryService::new(
        household_access_policy.clone(),
        custom_entry_repository.clone(),
        household_events_publisher.clone(),
    ));

    let set_shopping_quantity_service = Arc::new(SetShoppingQuantityService::new(
        household_access_policy.clone(),
        inventory_item_repository.clone(),
        shopping_state_repository.clone(),
        household_events_publisher.clone(),
    ));

    let set_checked_service = Arc::new(SetCheckedService::new(
        household_access_policy.clone(),
        inventory_item_repository.clone(),
        shopping_state_repository.clone(),
        household_events_publisher.clone(),
    ));

    let set_note_service = Arc::new(SetNoteService::new(
        household_access_policy.clone(),
        inventory_item_repository.clone(),
        shopping_state_repository.clone(),
        household_events_publisher.clone(),
    ));

    let dismiss_shopping_item_service = Arc::new(DismissShoppingItemService::new(
        household_access_policy.clone(),
        inventory_item_repository.clone(),
        shopping_state_repository.clone(),
        household_events_publisher.clone(),
    ));

    ShoppingState {
        list_shopping: list_shopping_service,
        create_custom_shopping_entry: create_custom_shopping_entry_service,
        delete_custom_shopping_entry: delete_custom_shopping_entry_service,
        set_custom_shopping_entry_checked: set_custom_shopping_entry_checked_service,
        update_custom_shopping_entry: update_custom_shopping_entry_service,
        set_shopping_quantity: set_shopping_quantity_service,
        set_checked: set_checked_service,
        set_note: set_note_service,
        dismiss_shopping_item: dismiss_shopping_item_service,
    }
}
