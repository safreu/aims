use backend::modules::{
    accounts::{adapters::PostgresUserRepository, ports::UserRepository},
    households::{adapters::PostgresHouseholdRepository, domain::HouseholdKind},
    inventory::{
        adapters::{PostgresCategoryRepository, PostgresInventoryItemRepository},
        ports::{CategoryRepository, InventoryItemRepository},
    },
    shopping::{
        adapters::{PostgresInventoryShoppingStateRepository, PostgresShoppingListQuery},
        domain::InventoryShoppingState,
        ports::{InventoryShoppingStateRepository, ShoppingListQuery},
        read_models::InventoryShoppingCategory,
    },
};
use chrono::Utc;
use sqlx::PgPool;

use crate::integration::{
    builders::{CategoryTestBuilder, InventoryItemTestBuilder, UserTestBuilder},
    helpers::insert_owned_household,
};

#[sqlx::test]
async fn item_with_positive_calculated_quantity_appears(pool: PgPool) {
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let shopping_list_query = PostgresShoppingListQuery::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .name("Tofu")
        .current_stock(1)
        .reorder_threshold(5)
        .build();

    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let result = shopping_list_query
        .list_inventory_entries(&household.id())
        .await
        .expect("Inventory entries lookup should succeed");

    assert_eq!(result[0].item_id, item.id());
    assert_eq!(result[0].name, item.name().as_str());
    assert_eq!(
        result[0].quantity,
        item.shopping_quantity()
            .expect("Calculation of shopping quantity should succeed")
    );
}

#[sqlx::test]
async fn item_above_reorder_threshold_does_not_appear(pool: PgPool) {
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let shopping_list_query = PostgresShoppingListQuery::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .name("Tofu")
        .current_stock(5)
        .reorder_threshold(1)
        .build();

    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let result = shopping_list_query
        .list_inventory_entries(&household.id())
        .await
        .expect("Inventory entries lookup should succeed");

    assert!(result.is_empty())
}

#[sqlx::test]
async fn quantity_override_replaces_calculated_quantity(pool: PgPool) {
    let shopping_state_repository = PostgresInventoryShoppingStateRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let shopping_list_query = PostgresShoppingListQuery::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .name("Tofu")
        .current_stock(1)
        .reorder_threshold(2)
        .build();

    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let mut state = InventoryShoppingState::new(household.id(), item.id());
    state
        .set_quantity_override(10)
        .expect("Setting quantity override should succeed");

    shopping_state_repository
        .upsert(&state)
        .await
        .expect("Upserting should succeed");

    let result = shopping_list_query
        .list_inventory_entries(&household.id())
        .await
        .expect("Inventory entries lookup should succeed");

    assert_eq!(result[0].item_id, item.id());
    assert_eq!(result[0].name, item.name().as_str());
    assert_eq!(result[0].quantity, 10);
}

#[sqlx::test]
async fn quantity_override_can_make_item_appear_above_threshold(pool: PgPool) {
    let shopping_state_repository = PostgresInventoryShoppingStateRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let shopping_list_query = PostgresShoppingListQuery::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .name("Tofu")
        .current_stock(2)
        .reorder_threshold(1)
        .build();

    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let mut state = InventoryShoppingState::new(household.id(), item.id());
    state
        .set_quantity_override(10)
        .expect("Setting quantity override should succeed");

    shopping_state_repository
        .upsert(&state)
        .await
        .expect("Upserting should succeed");

    let result = shopping_list_query
        .list_inventory_entries(&household.id())
        .await
        .expect("Inventory entries lookup should succeed");

    assert_eq!(result[0].item_id, item.id());
    assert_eq!(result[0].name, item.name().as_str());
    assert_eq!(result[0].quantity, 10);
}

#[sqlx::test]
async fn dismissed_item_does_not_appear(pool: PgPool) {
    let shopping_state_repository = PostgresInventoryShoppingStateRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let shopping_list_query = PostgresShoppingListQuery::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .name("Tofu")
        .current_stock(1)
        .reorder_threshold(2)
        .build();

    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let mut state = InventoryShoppingState::new(household.id(), item.id());
    state.dismiss();

    shopping_state_repository
        .upsert(&state)
        .await
        .expect("Upserting should succeed");

    let result = shopping_list_query
        .list_inventory_entries(&household.id())
        .await
        .expect("Inventory entries lookup should succeed");

    assert!(result.is_empty());
}

#[sqlx::test]
async fn shopping_state_note_and_checked_state_are_returned(pool: PgPool) {
    let shopping_state_repository = PostgresInventoryShoppingStateRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let shopping_list_query = PostgresShoppingListQuery::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .name("Tofu")
        .current_stock(1)
        .reorder_threshold(2)
        .build();

    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let mut state = InventoryShoppingState::new(household.id(), item.id());
    state.check();
    state
        .set_note(Some("this is a note".to_owned()))
        .expect("Setting note should succeed");

    shopping_state_repository
        .upsert(&state)
        .await
        .expect("Upserting should succeed");

    let result = shopping_list_query
        .list_inventory_entries(&household.id())
        .await
        .expect("Inventory entries lookup should succeed");

    assert_eq!(result[0].note, Some("this is a note".to_owned()));
    assert!(result[0].checked);
    assert_eq!(
        result[0].quantity,
        item.shopping_quantity()
            .expect("Shopping quantity should succeed")
    );
}

#[sqlx::test]
async fn category_is_returned(pool: PgPool) {
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let shopping_list_query = PostgresShoppingListQuery::new(pool.clone());
    let category_repository = PostgresCategoryRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let category = CategoryTestBuilder::new(household.id())
        .name("Food")
        .build();

    category_repository
        .insert(&category)
        .await
        .expect("Category insertion should succeed");

    let item = InventoryItemTestBuilder::new(household.id())
        .name("Tofu")
        .current_stock(1)
        .reorder_threshold(5)
        .category(category.id())
        .build();

    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let result = shopping_list_query
        .list_inventory_entries(&household.id())
        .await
        .expect("Inventory entries lookup should succeed");

    assert_eq!(result[0].item_id, item.id());
    assert_eq!(result[0].name, item.name().as_str());
    assert_eq!(
        result[0].category,
        Some(InventoryShoppingCategory {
            id: category.id(),
            name: "Food".to_owned()
        })
    );
}

#[sqlx::test]
async fn archived_item_does_not_appear(pool: PgPool) {
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let shopping_list_query = PostgresShoppingListQuery::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let mut item = InventoryItemTestBuilder::new(household.id())
        .name("Tofu")
        .current_stock(5)
        .reorder_threshold(1)
        .build();

    item.archive(Utc::now())
        .expect("Inventory item archiving should succeed");

    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let result = shopping_list_query
        .list_inventory_entries(&household.id())
        .await
        .expect("Inventory entries lookup should succeed");

    assert!(result.is_empty())
}
