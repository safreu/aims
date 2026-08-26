use backend::modules::{
    accounts::{adapters::PostgresUserRepository, ports::UserRepository},
    households::{
        adapters::PostgresHouseholdRepository,
        domain::{HouseholdId, HouseholdKind},
    },
    inventory::domain::InventoryPriority,
    shopping::{
        adapters::PostgresCustomShoppingEntryRepository,
        domain::{CustomShoppingEntry, CustomShoppingEntryId, CustomShoppingEntryTitle},
        ports::{CustomShoppingEntryRepository, CustomShoppingEntryRepositoryError},
    },
};
use chrono::{SubsecRound, Utc};
use sqlx::PgPool;

use crate::integration::{builders::UserTestBuilder, helpers::insert_owned_household};

#[sqlx::test]
async fn custom_entry_can_be_inserted_and_loaded(pool: PgPool) {
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let custom_shopping_entry_repository = PostgresCustomShoppingEntryRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let entry = CustomShoppingEntry::new(
        CustomShoppingEntryId::new(),
        household.id(),
        CustomShoppingEntryTitle::parse("Tofu")
            .expect("Custom shopping entry name should be valid"),
        5,
        backend::modules::inventory::domain::InventoryPriority::Default,
        None,
        Utc::now().trunc_subsecs(6),
    )
    .expect("Custom shopping entry should be valid");

    custom_shopping_entry_repository
        .insert(&entry)
        .await
        .expect("Custom shopping entry insertion should succeed");

    let stored = custom_shopping_entry_repository
        .find_by_id_for_household(&entry.id(), &household.id())
        .await
        .expect("Custom shopping entry lookup should succeed")
        .expect("Custom shopping entry should succeed");

    assert_eq!(stored, entry)
}

#[sqlx::test]
async fn custom_entry_can_be_updated(pool: PgPool) {
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let custom_shopping_entry_repository = PostgresCustomShoppingEntryRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let mut entry = CustomShoppingEntry::new(
        CustomShoppingEntryId::new(),
        household.id(),
        CustomShoppingEntryTitle::parse("Tofu")
            .expect("Custom shopping entry name should be valid"),
        5,
        backend::modules::inventory::domain::InventoryPriority::Default,
        None,
        Utc::now().trunc_subsecs(6),
    )
    .expect("Custom shopping entry should be valid");

    custom_shopping_entry_repository
        .insert(&entry)
        .await
        .expect("Custom shopping entry insertion should succeed");

    entry
        .rename(
            CustomShoppingEntryTitle::parse("Tofu")
                .expect("Custom shopping entry name should be valid"),
            Utc::now().trunc_subsecs(6),
        )
        .expect("Custom shopping entry update should be valid");

    custom_shopping_entry_repository
        .update(&entry)
        .await
        .expect("Custom shopping entry update should succeed");

    let stored = custom_shopping_entry_repository
        .find_by_id_for_household(&entry.id(), &household.id())
        .await
        .expect("Custom shopping entry lookup should succeed")
        .expect("Custom shopping entry should succeed");

    assert_eq!(stored, entry)
}

#[sqlx::test]
async fn custom_entry_can_be_deleted(pool: PgPool) {
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let custom_shopping_entry_repository = PostgresCustomShoppingEntryRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let entry = CustomShoppingEntry::new(
        CustomShoppingEntryId::new(),
        household.id(),
        CustomShoppingEntryTitle::parse("Tofu")
            .expect("Custom shopping entry name should be valid"),
        5,
        backend::modules::inventory::domain::InventoryPriority::Default,
        None,
        Utc::now(),
    )
    .expect("Custom shopping entry should be valid");

    custom_shopping_entry_repository
        .insert(&entry)
        .await
        .expect("Custom shopping entry insertion should succeed");

    custom_shopping_entry_repository
        .delete(&entry.id(), &household.id())
        .await
        .expect("Custom shopping entry deletion should succeed");

    let stored = custom_shopping_entry_repository
        .find_by_id_for_household(&entry.id(), &household.id())
        .await
        .expect("Custom shopping entry lookup should succeed");

    assert!(stored.is_none())
}

#[sqlx::test]
async fn missing_entry_returns_none(pool: PgPool) {
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let custom_shopping_entry_repository = PostgresCustomShoppingEntryRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let entry = CustomShoppingEntry::new(
        CustomShoppingEntryId::new(),
        household.id(),
        CustomShoppingEntryTitle::parse("Tofu")
            .expect("Custom shopping entry name should be valid"),
        5,
        backend::modules::inventory::domain::InventoryPriority::Default,
        None,
        Utc::now(),
    )
    .expect("Custom shopping entry should be valid");

    let stored = custom_shopping_entry_repository
        .find_by_id_for_household(&entry.id(), &household.id())
        .await
        .expect("Custom shopping entry lookup should succeed");

    assert!(stored.is_none())
}

#[sqlx::test]
async fn update_missing_entry_returns_not_found(pool: PgPool) {
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let custom_shopping_entry_repository = PostgresCustomShoppingEntryRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let mut entry = CustomShoppingEntry::new(
        CustomShoppingEntryId::new(),
        household.id(),
        CustomShoppingEntryTitle::parse("Tofu")
            .expect("Custom shopping entry name should be valid"),
        5,
        backend::modules::inventory::domain::InventoryPriority::Default,
        None,
        Utc::now(),
    )
    .expect("Custom shopping entry should be valid");

    entry
        .set_priority(InventoryPriority::High, Utc::now())
        .expect("Custom shopping entry update should be valid");

    let result = custom_shopping_entry_repository.update(&entry).await;

    assert_eq!(
        result,
        Err(CustomShoppingEntryRepositoryError::EntryNotFound)
    )
}

#[sqlx::test]
async fn delete_missing_entry_returns_not_found(pool: PgPool) {
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let custom_shopping_entry_repository = PostgresCustomShoppingEntryRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let entry = CustomShoppingEntry::new(
        CustomShoppingEntryId::new(),
        household.id(),
        CustomShoppingEntryTitle::parse("Tofu")
            .expect("Custom shopping entry name should be valid"),
        5,
        backend::modules::inventory::domain::InventoryPriority::Default,
        None,
        Utc::now(),
    )
    .expect("Custom shopping entry should be valid");

    let result = custom_shopping_entry_repository
        .delete(&entry.id(), &household.id())
        .await;

    assert_eq!(
        result,
        Err(CustomShoppingEntryRepositoryError::EntryNotFound)
    )
}

#[sqlx::test]
async fn entries_are_scoped_to_household(pool: PgPool) {
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let custom_shopping_entry_repository = PostgresCustomShoppingEntryRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let entry = CustomShoppingEntry::new(
        CustomShoppingEntryId::new(),
        household.id(),
        CustomShoppingEntryTitle::parse("Tofu")
            .expect("Custom shopping entry name should be valid"),
        5,
        backend::modules::inventory::domain::InventoryPriority::Default,
        None,
        Utc::now(),
    )
    .expect("Custom shopping entry should be valid");

    custom_shopping_entry_repository
        .insert(&entry)
        .await
        .expect("Custom shopping entry insertion should succeed");

    let stored = custom_shopping_entry_repository
        .find_for_household(&HouseholdId::new())
        .await
        .expect("Custom shopping entry lookup should succeed");

    assert!(stored.is_empty())
}
