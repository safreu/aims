use backend::modules::{
    accounts::{adapters::PostgresUserRepository, ports::UserRepository},
    devices::{
        adapters::PostgresDeviceRepository,
        domain::DeviceName,
        ports::{DeviceRepository, DeviceRepositoryError},
    },
    households::{
        adapters::PostgresHouseholdRepository,
        domain::{HouseholdId, HouseholdKind},
    },
};
use chrono::{SubsecRound, Utc};
use sqlx::PgPool;

use crate::integration::{
    builders::{DeviceTestBuilder, UserTestBuilder},
    helpers::insert_owned_household,
};

#[sqlx::test]
async fn device_can_be_inserted_and_loaded(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let device_repository = PostgresDeviceRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let device = DeviceTestBuilder::new(household.id()).build();

    let result = device_repository.insert(&device).await;

    assert!(result.is_ok());

    let stored = device_repository
        .find_by_id(&device.id(), &household.id())
        .await
        .expect("Device lookup should succeed")
        .expect("Device should exist");

    assert_eq!(stored.id(), device.id());
    assert_eq!(stored.household_id(), device.household_id());
    assert_eq!(stored.name(), device.name());
    assert_eq!(stored.created_at(), device.created_at());
    assert_eq!(stored.kind(), device.kind());
    assert_eq!(stored.updated_at(), device.updated_at());
    assert_eq!(stored.revoked_at(), device.revoked_at());
}

#[sqlx::test]
async fn active_devices_for_household_are_returned(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let device_repository = PostgresDeviceRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let device = DeviceTestBuilder::new(household.id()).build();

    device_repository
        .insert(&device)
        .await
        .expect("Device insertion should succeed");

    let stored = device_repository
        .find_active_for_household(&household.id())
        .await
        .expect("Device lookup should succeed");

    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0], device);
}

#[sqlx::test]
async fn revoked_devices_are_not_returned_as_active(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let device_repository = PostgresDeviceRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let mut device = DeviceTestBuilder::new(household.id()).build();

    device
        .revoke(Utc::now())
        .expect("Device revoking should succeed");

    device_repository
        .insert(&device)
        .await
        .expect("Device insertion should succeed");

    let stored = device_repository
        .find_active_for_household(&household.id())
        .await
        .expect("Device lookup should succeed");

    assert_eq!(stored.len(), 0);
    assert!(!stored.contains(&device));
}

#[sqlx::test]
async fn devices_from_other_households_are_not_returned(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let device_repository = PostgresDeviceRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let device = DeviceTestBuilder::new(household.id()).build();

    device_repository
        .insert(&device)
        .await
        .expect("Device insertion should succeed");

    let stored = device_repository
        .find_active_for_household(&HouseholdId::new())
        .await
        .expect("Device lookup should succeed");

    assert_eq!(stored.len(), 0);
    assert!(!stored.contains(&device));
}

#[sqlx::test]
async fn device_can_be_updated(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let device_repository = PostgresDeviceRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let mut device = DeviceTestBuilder::new(household.id()).build();

    device_repository
        .insert(&device)
        .await
        .expect("Device insertion should succeed");

    let now = Utc::now().trunc_subsecs(6);

    device
        .rename(
            DeviceName::parse("New Scanner").expect("Device name should be valid"),
            now,
        )
        .expect("Device renaming should succeed");

    device_repository
        .update(&device)
        .await
        .expect("Device update should succeed");

    let stored = device_repository
        .find_by_id(&device.id(), &household.id())
        .await
        .expect("Device lookup should succeed")
        .expect("Device should exist");

    assert_eq!(stored.id(), device.id());
    assert_eq!(stored.household_id(), device.household_id());
    assert_eq!(stored.name().as_str(), "New Scanner");
    assert_eq!(stored.created_at(), device.created_at());
    assert_eq!(stored.kind(), device.kind());
    assert_eq!(stored.updated_at(), now);
    assert_eq!(stored.revoked_at(), device.revoked_at());
}

#[sqlx::test]
async fn updating_unknown_device_returns_not_found(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let device_repository = PostgresDeviceRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let device = DeviceTestBuilder::new(household.id()).build();
    let mut another_device = DeviceTestBuilder::new(household.id()).build();

    device_repository
        .insert(&device)
        .await
        .expect("Device insertion should succeed");

    let now = Utc::now().trunc_subsecs(6);

    another_device
        .rename(
            DeviceName::parse("New Scanner").expect("Device name should be valid"),
            now,
        )
        .expect("Device renaming should succeed");

    let result = device_repository.update(&another_device).await;

    assert_eq!(result, Err(DeviceRepositoryError::DeviceNotFound))
}
