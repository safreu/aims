use backend::modules::{
    accounts::{adapters::PostgresUserRepository, ports::UserRepository},
    households::{
        adapters::PostgresHouseholdRepository,
        domain::{HouseholdId, HouseholdKind},
    },
    inventory::{
        adapters::PostgresInventoryItemRepository, domain::InventoryItemId,
        ports::InventoryItemRepository,
    },
    shopping::{
        adapters::PostgresInventoryShoppingStateRepository, domain::InventoryShoppingState,
        ports::InventoryShoppingStateRepository,
    },
};
use sqlx::PgPool;

use crate::integration::{
    builders::{InventoryItemTestBuilder, UserTestBuilder},
    helpers::insert_owned_household,
};

#[sqlx::test]
async fn shopping_state_can_be_upserted_and_loaded(pool: PgPool) {
    let shopping_state_repository = PostgresInventoryShoppingStateRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());

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

    let mut state = InventoryShoppingState::new(household.id(), item.id());

    state
        .set_quantity_override(5)
        .expect("Quantity should be valid");

    state
        .set_note(Some("Tasty".to_owned()))
        .expect("Note should be valid");

    state.check();
    state.dismiss();

    shopping_state_repository
        .upsert(&state)
        .await
        .expect("Upserting shopping state should succeed");

    let stored = shopping_state_repository
        .find_by_item(&household.id(), &item.id())
        .await
        .expect("Shopping state lookup should succeed")
        .expect("Shopping state should exists");

    assert_eq!(stored.household_id(), household.id());
    assert_eq!(stored.item_id(), item.id());
    assert_eq!(stored.quantity_override(), Some(5));
    assert_eq!(stored.note(), Some("Tasty"));
    assert!(stored.checked());
    assert!(stored.dismissed())
}

#[sqlx::test]
async fn existing_shopping_state_is_updated_by_upsert(pool: PgPool) {
    let shopping_state_repository = PostgresInventoryShoppingStateRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());

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

    let mut state = InventoryShoppingState::new(household.id(), item.id());

    state
        .set_quantity_override(5)
        .expect("Quantity should be valid");

    shopping_state_repository
        .upsert(&state)
        .await
        .expect("First upserting shopping state should succeed");

    state
        .set_quantity_override(2)
        .expect("Quantity should be valid");
    state
        .set_note(Some("Tasty".to_owned()))
        .expect("Note should be valid");
    state.check();

    shopping_state_repository
        .upsert(&state)
        .await
        .expect("Second upserting shopping state should succeed");

    let stored = shopping_state_repository
        .find_by_item(&household.id(), &item.id())
        .await
        .expect("Shopping state lookup should succeed")
        .expect("Shopping state should exists");

    assert_eq!(stored.quantity_override(), Some(2));
    assert_eq!(stored.note(), Some("Tasty"));
    assert!(stored.checked());
}

#[sqlx::test]
async fn missing_shopping_state_returns_none(pool: PgPool) {
    let repository = PostgresInventoryShoppingStateRepository::new(pool);

    let result = repository
        .find_by_item(&HouseholdId::new(), &InventoryItemId::new())
        .await
        .expect("Finding shopping state should succeed");

    assert!(result.is_none())
}

#[sqlx::test]
async fn shopping_state_can_be_deleted(pool: PgPool) {
    let shopping_state_repository = PostgresInventoryShoppingStateRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());

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

    let state = InventoryShoppingState::new(household.id(), item.id());

    shopping_state_repository
        .upsert(&state)
        .await
        .expect("Upserting shopping state should succeed");

    shopping_state_repository
        .delete(&household.id(), &item.id())
        .await
        .expect("Deleting shopping state should succeed");

    let result = shopping_state_repository
        .find_by_item(&household.id(), &item.id())
        .await
        .expect("Shopping state lookup should succeed");

    assert!(result.is_none())
}

#[sqlx::test]
async fn deleting_missing_shopping_state_succeeds(pool: PgPool) {
    let shopping_state_repository = PostgresInventoryShoppingStateRepository::new(pool.clone());

    let result = shopping_state_repository
        .delete(&HouseholdId::new(), &InventoryItemId::new())
        .await;

    assert!(result.is_ok())
}
