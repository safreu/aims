use std::sync::Arc;

use backend::modules::{
    accounts::{adapters::PostgresUserRepository, domain::UserId, ports::UserRepository},
    households::{adapters::PostgresHouseholdRepository, domain::HouseholdKind},
    inventory::{
        self,
        adapters::{PostgresInventoryItemRepository, PostgresInventoryStockRepository},
        domain::InventoryItemId,
        ports::{InventoryItemRepository, InventoryStockRepository, StockMutationContext},
    },
};
use chrono::Utc;
use sqlx::PgPool;

use crate::integration::{
    builders::{InventoryItemTestBuilder, UserTestBuilder},
    helpers::insert_owned_household,
};

fn manual_stock_context(user_id: UserId) -> StockMutationContext {
    StockMutationContext {
        actor_user_id: Some(user_id),
        actor_device_id: None,
        source: inventory::domain::InventoryStockEventSource::Manual,
    }
}

#[sqlx::test]
async fn stock_can_be_increased(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = Arc::new(PostgresInventoryStockRepository::new(pool.clone()));

    let owner = UserTestBuilder::new().build();
    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(0)
        .name("Tofu".to_owned())
        .build();
    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    stock_repository
        .increase(
            &household.id(),
            &item.id(),
            1,
            &manual_stock_context(owner.id()),
            Utc::now(),
        )
        .await
        .expect("Inventory item stock increase should succeed");

    let stored = inventory_item_repository
        .find_by_id(&item.id(), &household.id())
        .await
        .expect("Inventory item lookup should succeed")
        .expect("Inventory item should exists");

    assert_eq!(stored.current_stock(), 1)
}

#[sqlx::test]
async fn stock_can_be_decreased(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = Arc::new(PostgresInventoryStockRepository::new(pool.clone()));

    let owner = UserTestBuilder::new().build();
    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(1)
        .name("Tofu".to_owned())
        .build();
    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    stock_repository
        .decrease(
            &household.id(),
            &item.id(),
            1,
            &manual_stock_context(owner.id()),
            Utc::now(),
        )
        .await
        .expect("Inventory item stock decrease should succeed");

    let stored = inventory_item_repository
        .find_by_id(&item.id(), &household.id())
        .await
        .expect("Inventory item lookup should succeed")
        .expect("Inventory item should exists");

    assert_eq!(stored.current_stock(), 0)
}

#[sqlx::test]
async fn stock_can_be_set(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = Arc::new(PostgresInventoryStockRepository::new(pool.clone()));

    let owner = UserTestBuilder::new().build();
    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(1)
        .name("Tofu".to_owned())
        .build();
    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    stock_repository
        .set(
            &household.id(),
            &item.id(),
            10,
            &manual_stock_context(owner.id()),
            Utc::now(),
        )
        .await
        .expect("Inventory item stock decrease should succeed");

    let stored = inventory_item_repository
        .find_by_id(&item.id(), &household.id())
        .await
        .expect("Inventory item lookup should succeed")
        .expect("Inventory item should exists");

    assert_eq!(stored.current_stock(), 10)
}

#[sqlx::test]
async fn decreasing_below_zero_is_rejected(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = Arc::new(PostgresInventoryStockRepository::new(pool.clone()));

    let owner = UserTestBuilder::new().build();
    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(0)
        .name("Tofu".to_owned())
        .build();
    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let result = stock_repository
        .decrease(
            &household.id(),
            &item.id(),
            1,
            &manual_stock_context(owner.id()),
            Utc::now(),
        )
        .await;

    assert_eq!(
        result,
        Err(inventory::ports::InventoryStockRepositoryError::InsufficientStock)
    )
}

#[sqlx::test]
async fn increasing_above_u32_max_is_rejected(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = Arc::new(PostgresInventoryStockRepository::new(pool.clone()));

    let owner = UserTestBuilder::new().build();
    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(u32::MAX)
        .name("Tofu".to_owned())
        .build();
    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let result = stock_repository
        .increase(
            &household.id(),
            &item.id(),
            1,
            &manual_stock_context(owner.id()),
            Utc::now(),
        )
        .await;

    assert_eq!(
        result,
        Err(inventory::ports::InventoryStockRepositoryError::StockOverflow)
    )
}

#[sqlx::test]
async fn archived_item_cannot_be_modified(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = Arc::new(PostgresInventoryStockRepository::new(pool.clone()));

    let owner = UserTestBuilder::new().build();
    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let mut item = InventoryItemTestBuilder::new(household.id())
        .current_stock(0)
        .name("Tofu".to_owned())
        .build();
    item.archive(Utc::now())
        .expect("Item archiving should succeed");
    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let result = stock_repository
        .increase(
            &household.id(),
            &item.id(),
            1,
            &manual_stock_context(owner.id()),
            Utc::now(),
        )
        .await;

    assert_eq!(
        result,
        Err(inventory::ports::InventoryStockRepositoryError::ItemArchived)
    )
}

#[sqlx::test]
async fn unknown_item_returns_not_found(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = Arc::new(PostgresInventoryStockRepository::new(pool.clone()));

    let owner = UserTestBuilder::new().build();
    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(1)
        .name("Tofu".to_owned())
        .build();
    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let result = stock_repository
        .decrease(
            &household.id(),
            &InventoryItemId::new(),
            1,
            &manual_stock_context(owner.id()),
            Utc::now(),
        )
        .await;

    assert_eq!(
        result,
        Err(inventory::ports::InventoryStockRepositoryError::ItemNotFound)
    )
}

#[sqlx::test]
async fn item_from_different_household_returns_not_found(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = Arc::new(PostgresInventoryStockRepository::new(pool.clone()));

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
        .current_stock(1)
        .name("Tofu".to_owned())
        .build();
    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let result = stock_repository
        .decrease(
            &another_household.id(),
            &item.id(),
            1,
            &manual_stock_context(owner.id()),
            Utc::now(),
        )
        .await;

    assert_eq!(
        result,
        Err(inventory::ports::InventoryStockRepositoryError::ItemNotFound)
    )
}

#[sqlx::test]
async fn concurrent_stock_increases_are_atomic(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = Arc::new(PostgresInventoryStockRepository::new(pool.clone()));

    let owner = UserTestBuilder::new().build();
    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(0)
        .name("Tofu".to_owned())
        .build();
    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let context = manual_stock_context(owner.id());

    let mut tasks = Vec::new();

    for _ in 0..100 {
        let repository = stock_repository.clone();
        let household_id = household.id();
        let item_id = item.id();
        let context = context.clone();

        tasks.push(tokio::spawn(async move {
            repository
                .increase(&household_id, &item_id, 1, &context, Utc::now())
                .await
        }));
    }

    for task in tasks {
        task.await
            .expect("Stock increase task should not panic")
            .expect("Stock increase should succeed");
    }

    let stored = inventory_item_repository
        .find_by_id(&item.id(), &household.id())
        .await
        .expect("Inventory item lookup should succeed")
        .expect("Inventory item should exists");

    assert_eq!(stored.current_stock(), 100)
}

#[sqlx::test]
async fn increasing_stock_creates_stock_event(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = Arc::new(PostgresInventoryStockRepository::new(pool.clone()));

    let owner = UserTestBuilder::new().build();
    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(3)
        .name("Tofu".to_owned())
        .build();
    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let context = manual_stock_context(owner.id());

    stock_repository
        .increase(&household.id(), &item.id(), 2, &context, Utc::now())
        .await
        .expect("Stock increase should succeed");

    let event = sqlx::query!(
        r#"
           SELECT
                household_id,
                item_id,
                actor_user_id,
                actor_device_id,
                kind,
                source,
                amount,
                stock_before,
                stock_after
            FROM inventory_stock_events
            WHERE item_id = $1
           "#,
        item.id().into_uuid(),
    )
    .fetch_one(&pool)
    .await
    .expect("Stock event should exists");

    assert_eq!(event.household_id, household.id().into_uuid());
    assert_eq!(event.item_id, item.id().into_uuid());
    assert_eq!(event.actor_user_id, Some(owner.id().into_uuid()));
    assert_eq!(event.actor_device_id, None);
    assert_eq!(event.kind, "increase");
    assert_eq!(event.source, "manual");
    assert_eq!(event.amount, Some(2));
    assert_eq!(event.stock_before, 3);
    assert_eq!(event.stock_after, 5)
}

#[sqlx::test]
async fn decreasing_stock_creates_stock_event(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = Arc::new(PostgresInventoryStockRepository::new(pool.clone()));

    let owner = UserTestBuilder::new().build();
    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(3)
        .name("Tofu".to_owned())
        .build();
    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let context = manual_stock_context(owner.id());

    stock_repository
        .decrease(&household.id(), &item.id(), 2, &context, Utc::now())
        .await
        .expect("Stock decrease should succeed");

    let event = sqlx::query!(
        r#"
        SELECT
             household_id,
             item_id,
             actor_user_id,
             actor_device_id,
             kind,
             source,
             amount,
             stock_before,
             stock_after
         FROM inventory_stock_events
         WHERE item_id = $1
        "#,
        item.id().into_uuid(),
    )
    .fetch_one(&pool)
    .await
    .expect("Stock event should exists");

    assert_eq!(event.household_id, household.id().into_uuid());
    assert_eq!(event.item_id, item.id().into_uuid());
    assert_eq!(event.actor_user_id, Some(owner.id().into_uuid()));
    assert_eq!(event.actor_device_id, None);
    assert_eq!(event.kind, "decrease");
    assert_eq!(event.source, "manual");
    assert_eq!(event.amount, Some(2));
    assert_eq!(event.stock_before, 3);
    assert_eq!(event.stock_after, 1)
}

#[sqlx::test]
async fn setting_stock_creates_stock_event(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_item_repository = PostgresInventoryItemRepository::new(pool.clone());
    let stock_repository = Arc::new(PostgresInventoryStockRepository::new(pool.clone()));

    let owner = UserTestBuilder::new().build();
    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .current_stock(3)
        .name("Tofu".to_owned())
        .build();
    inventory_item_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let context = manual_stock_context(owner.id());

    stock_repository
        .set(&household.id(), &item.id(), 2, &context, Utc::now())
        .await
        .expect("Stock decrease should succeed");

    let event = sqlx::query!(
        r#"
        SELECT
             household_id,
             item_id,
             actor_user_id,
             actor_device_id,
             kind,
             source,
             amount,
             stock_before,
             stock_after
         FROM inventory_stock_events
         WHERE item_id = $1
        "#,
        item.id().into_uuid(),
    )
    .fetch_one(&pool)
    .await
    .expect("Stock event should exists");

    assert_eq!(event.household_id, household.id().into_uuid());
    assert_eq!(event.item_id, item.id().into_uuid());
    assert_eq!(event.actor_user_id, Some(owner.id().into_uuid()));
    assert_eq!(event.actor_device_id, None);
    assert_eq!(event.kind, "set");
    assert_eq!(event.source, "manual");
    assert_eq!(event.amount, None);
    assert_eq!(event.stock_before, 3);
    assert_eq!(event.stock_after, 2)
}
