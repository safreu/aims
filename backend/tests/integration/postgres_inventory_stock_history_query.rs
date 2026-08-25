use std::panic;

use backend::modules::{
    accounts::{adapters::PostgresUserRepository, ports::UserRepository},
    households::{adapters::PostgresHouseholdRepository, domain::HouseholdKind},
    inventory::{
        adapters::{
            PostgresInventoryItemRepository, PostgresInventoryStockHistoryQuery,
            PostgresInventoryStockRepository,
        },
        domain::{InventoryItemId, InventoryStockEventKind, InventoryStockEventSource},
        ports::{
            InventoryItemRepository, InventoryStockHistoryQuery, InventoryStockRepository,
            StockMutationContext,
        },
        read_models::InventoryStockHistoryActor,
    },
};
use chrono::Utc;
use sqlx::PgPool;

use crate::integration::{
    builders::{InventoryItemTestBuilder, UserTestBuilder},
    helpers::insert_owned_household,
};

#[sqlx::test]
async fn stock_history_is_returned_newest_first(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = PostgresInventoryStockRepository::new(pool.clone());
    let history_query = PostgresInventoryStockHistoryQuery::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(5)
        .name("Tofu".to_owned())
        .build();

    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let context = StockMutationContext {
        actor_user_id: Some(owner.id()),
        actor_device_id: None,
        source: InventoryStockEventSource::Manual,
    };

    stock_repository
        .increase(&household.id(), &item.id(), 2, &context, Utc::now())
        .await
        .expect("Stock increase should succeed");

    stock_repository
        .decrease(&household.id(), &item.id(), 1, &context, Utc::now())
        .await
        .expect("Stock decrease should succeed");

    let history = history_query
        .find_for_item(&household.id(), &item.id())
        .await
        .expect("Stock history query should succeed");

    assert_eq!(history.len(), 2);
    assert_eq!(history[0].kind, InventoryStockEventKind::Decrease);
    assert_eq!(history[1].kind, InventoryStockEventKind::Increase);
    assert!(history[0].sequence_number > history[1].sequence_number);
}

#[sqlx::test]
async fn user_actor_is_resolved(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = PostgresInventoryStockRepository::new(pool.clone());
    let history_query = PostgresInventoryStockHistoryQuery::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(5)
        .name("Tofu".to_owned())
        .build();

    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let context = StockMutationContext {
        actor_user_id: Some(owner.id()),
        actor_device_id: None,
        source: InventoryStockEventSource::Manual,
    };

    stock_repository
        .increase(&household.id(), &item.id(), 1, &context, Utc::now())
        .await
        .expect("Stock increase should succeed");

    let history = history_query
        .find_for_item(&household.id(), &item.id())
        .await
        .expect("Stock history query should succeed");

    assert_eq!(history.len(), 1);
    match &history[0].actor {
        InventoryStockHistoryActor::User { id, display_name } => {
            assert_eq!(*id, owner.id());
            assert_eq!(display_name.as_str(), owner.display_name().as_str());
        }
        _ => panic!("Expected user actor"),
    }
}

#[sqlx::test]
async fn device_actor_is_resolved(_pool: PgPool) {
    //TODO: After implementing the device repository, implement this test
}

#[sqlx::test]
async fn system_actor_is_resolved(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = PostgresInventoryStockRepository::new(pool.clone());
    let history_query = PostgresInventoryStockHistoryQuery::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(5)
        .name("Tofu".to_owned())
        .build();

    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let context = StockMutationContext {
        actor_user_id: None,
        actor_device_id: None,
        source: InventoryStockEventSource::System,
    };

    stock_repository
        .increase(&household.id(), &item.id(), 1, &context, Utc::now())
        .await
        .expect("Stock increase should succeed");

    let history = history_query
        .find_for_item(&household.id(), &item.id())
        .await
        .expect("Stock history query should succeed");

    assert_eq!(history.len(), 1);
    assert!(matches!(
        history[0].actor,
        InventoryStockHistoryActor::System
    ))
}

#[sqlx::test]
async fn history_from_other_household_is_not_returned(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = PostgresInventoryStockRepository::new(pool.clone());
    let history_query = PostgresInventoryStockHistoryQuery::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;
    let (another_household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(5)
        .name("Tofu".to_owned())
        .build();

    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let context = StockMutationContext {
        actor_user_id: Some(owner.id()),
        actor_device_id: None,
        source: InventoryStockEventSource::Manual,
    };

    stock_repository
        .increase(&household.id(), &item.id(), 2, &context, Utc::now())
        .await
        .expect("Stock increase should succeed");

    stock_repository
        .decrease(&household.id(), &item.id(), 1, &context, Utc::now())
        .await
        .expect("Stock decrease should succeed");

    let history = history_query
        .find_for_item(&another_household.id(), &item.id())
        .await
        .expect("Stock history query should succeed");

    assert!(history.is_empty())
}

#[sqlx::test]
async fn unknown_inventory_item_returns_not_found(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = PostgresInventoryStockRepository::new(pool.clone());
    let history_query = PostgresInventoryStockHistoryQuery::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(5)
        .name("Tofu".to_owned())
        .build();

    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let context = StockMutationContext {
        actor_user_id: Some(owner.id()),
        actor_device_id: None,
        source: InventoryStockEventSource::Manual,
    };

    stock_repository
        .increase(&household.id(), &item.id(), 2, &context, Utc::now())
        .await
        .expect("Stock increase should succeed");

    let history = history_query
        .find_for_item(&household.id(), &InventoryItemId::new())
        .await
        .expect("Stock history query should succeed");

    assert!(history.is_empty())
}

#[sqlx::test]
async fn inventory_item_without_history_returns_empty_list(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let history_query = PostgresInventoryStockHistoryQuery::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(5)
        .name("Tofu".to_owned())
        .build();

    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let history = history_query
        .find_for_item(&household.id(), &item.id())
        .await
        .expect("Stock history query should succeed");

    assert!(history.is_empty());
}
