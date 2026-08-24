use backend::modules::{
    accounts::{adapters::PostgresUserRepository, ports::UserRepository},
    devices::{
        adapters::{PostgresDeviceCredentialRepository, PostgresDeviceRepository},
        domain::{DeviceCredential, DeviceCredentialId, DeviceTokenHash},
        ports::{DeviceCredentialRepository, DeviceCredentialRepositoryError, DeviceRepository},
    },
    households::{adapters::PostgresHouseholdRepository, domain::HouseholdKind},
};
use chrono::{SubsecRound, Utc};
use sqlx::PgPool;

use crate::integration::{
    builders::{DeviceTestBuilder, UserTestBuilder},
    helpers::insert_owned_household,
};

#[sqlx::test]
async fn credential_can_be_inserted_and_loaded_by_device(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let device_repository = PostgresDeviceRepository::new(pool.clone());
    let credential_repository = PostgresDeviceCredentialRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let device = DeviceTestBuilder::new(household.id())
        .name("Scanner")
        .build();

    device_repository
        .insert(&device)
        .await
        .expect("Device insertion should succeed");

    let token_hash = DeviceTokenHash::from_encoded("test-device-token-hash")
        .expect("Device token hash should be valid");

    let credential = DeviceCredential::new(
        DeviceCredentialId::new(),
        device.id(),
        token_hash,
        Utc::now().trunc_subsecs(6),
    );

    credential_repository
        .insert(&credential)
        .await
        .expect("Device credential insertion should succeed");

    let stored = credential_repository
        .find_active_by_device_id(&device.id())
        .await
        .expect("Device credential lookup should succeed")
        .expect("Device credential should exists");

    assert_eq!(stored, credential);
}

#[sqlx::test]
async fn active_credential_can_be_loaded_by_token_hash(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let device_repository = PostgresDeviceRepository::new(pool.clone());
    let credential_repository = PostgresDeviceCredentialRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let device = DeviceTestBuilder::new(household.id())
        .name("Scanner")
        .build();

    device_repository
        .insert(&device)
        .await
        .expect("Device insertion should succeed");

    let token_hash = DeviceTokenHash::from_encoded("test-device-token-hash")
        .expect("Device token hash should be valid");

    let credential = DeviceCredential::new(
        DeviceCredentialId::new(),
        device.id(),
        token_hash,
        Utc::now().trunc_subsecs(6),
    );

    credential_repository
        .insert(&credential)
        .await
        .expect("Device credential insertion should succeed");

    let stored = credential_repository
        .find_active_by_token_hash(credential.token_hash())
        .await
        .expect("Device credential lookup should succeed")
        .expect("Device credential should exists");

    assert_eq!(stored, credential);
}

#[sqlx::test]
async fn second_active_credential_is_rejected(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let device_repository = PostgresDeviceRepository::new(pool.clone());
    let credential_repository = PostgresDeviceCredentialRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let device = DeviceTestBuilder::new(household.id())
        .name("Scanner")
        .build();

    device_repository
        .insert(&device)
        .await
        .expect("Device insertion should succeed");

    let token_hash = DeviceTokenHash::from_encoded("test-device-token-hash")
        .expect("Device token hash should be valid");

    let credential = DeviceCredential::new(
        DeviceCredentialId::new(),
        device.id(),
        token_hash,
        Utc::now().trunc_subsecs(6),
    );

    credential_repository
        .insert(&credential)
        .await
        .expect("Device credential insertion should succeed");

    let token_hash = DeviceTokenHash::from_encoded("test-device-token-hash")
        .expect("Device token hash should be valid");

    let another_credential = DeviceCredential::new(
        DeviceCredentialId::new(),
        device.id(),
        token_hash,
        Utc::now().trunc_subsecs(6),
    );

    let result = credential_repository.insert(&another_credential).await;

    assert_eq!(
        result,
        Err(DeviceCredentialRepositoryError::ActiveCredentialAlreadyExists)
    )
}

#[sqlx::test]
async fn active_credential_can_be_revoked(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let device_repository = PostgresDeviceRepository::new(pool.clone());
    let credential_repository = PostgresDeviceCredentialRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let device = DeviceTestBuilder::new(household.id())
        .name("Scanner")
        .build();

    device_repository
        .insert(&device)
        .await
        .expect("Device insertion should succeed");

    let token_hash = DeviceTokenHash::from_encoded("test-device-token-hash")
        .expect("Device token hash should be valid");

    let credential = DeviceCredential::new(
        DeviceCredentialId::new(),
        device.id(),
        token_hash,
        Utc::now().trunc_subsecs(6),
    );

    credential_repository
        .insert(&credential)
        .await
        .expect("Device credential insertion should succeed");

    credential_repository
        .revoke_active(&device.id(), Utc::now())
        .await
        .expect("Device revocation should succeed");

    let stored = credential_repository
        .find_active_by_device_id(&device.id())
        .await
        .expect("Device credential lookup should succeed");

    assert!(stored.is_none());
}

#[sqlx::test]
async fn revoked_credential_is_not_returned_as_active(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let device_repository = PostgresDeviceRepository::new(pool.clone());
    let credential_repository = PostgresDeviceCredentialRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let device = DeviceTestBuilder::new(household.id())
        .name("Scanner")
        .build();

    device_repository
        .insert(&device)
        .await
        .expect("Device insertion should succeed");

    let token_hash = DeviceTokenHash::from_encoded("test-device-token-hash")
        .expect("Device token hash should be valid");

    let credential = DeviceCredential::new(
        DeviceCredentialId::new(),
        device.id(),
        token_hash,
        Utc::now().trunc_subsecs(6),
    );

    credential_repository
        .insert(&credential)
        .await
        .expect("Device credential insertion should succeed");

    credential_repository
        .revoke_active(&device.id(), Utc::now())
        .await
        .expect("Device revocation should succeed");

    let stored = credential_repository
        .find_active_by_token_hash(credential.token_hash())
        .await
        .expect("Device credential lookup should succeed");

    assert!(stored.is_none());
}

#[sqlx::test]
async fn new_credential_can_be_created_after_previous_one_was_revoked(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let device_repository = PostgresDeviceRepository::new(pool.clone());
    let credential_repository = PostgresDeviceCredentialRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let device = DeviceTestBuilder::new(household.id())
        .name("Scanner")
        .build();

    device_repository
        .insert(&device)
        .await
        .expect("Device insertion should succeed");

    let token_hash = DeviceTokenHash::from_encoded("test-device-token-hash")
        .expect("Device token hash should be valid");

    let credential = DeviceCredential::new(
        DeviceCredentialId::new(),
        device.id(),
        token_hash,
        Utc::now().trunc_subsecs(6),
    );

    credential_repository
        .insert(&credential)
        .await
        .expect("Device credential insertion should succeed");

    credential_repository
        .revoke_active(&device.id(), Utc::now())
        .await
        .expect("Device revocation should succeed");

    let token_hash = DeviceTokenHash::from_encoded("another-test-device-token-hash")
        .expect("Device token hash should be valid");

    let another_credential = DeviceCredential::new(
        DeviceCredentialId::new(),
        device.id(),
        token_hash,
        Utc::now().trunc_subsecs(6),
    );

    let result = credential_repository.insert(&another_credential).await;

    assert!(result.is_ok());

    let stored = credential_repository
        .find_active_by_device_id(&device.id())
        .await
        .expect("Device credential lookup should succeed")
        .expect("Device credential should exists");

    assert_eq!(stored, another_credential);
}

#[sqlx::test]
async fn active_credential_can_be_rotated(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let device_repository = PostgresDeviceRepository::new(pool.clone());
    let credential_repository = PostgresDeviceCredentialRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let device = DeviceTestBuilder::new(household.id())
        .name("Scanner")
        .build();

    device_repository
        .insert(&device)
        .await
        .expect("Device insertion should succeed");

    let token_hash = DeviceTokenHash::from_encoded("test-device-token-hash")
        .expect("Device token hash should be valid");

    let credential = DeviceCredential::new(
        DeviceCredentialId::new(),
        device.id(),
        token_hash,
        Utc::now().trunc_subsecs(6),
    );

    credential_repository
        .insert(&credential)
        .await
        .expect("Device credential insertion should succeed");

    let token_hash = DeviceTokenHash::from_encoded("new-test-device-token-hash")
        .expect("Device token hash should be valid");

    let new_credential = DeviceCredential::new(
        DeviceCredentialId::new(),
        device.id(),
        token_hash,
        Utc::now().trunc_subsecs(6),
    );

    credential_repository
        .rotate(&device.id(), &new_credential, Utc::now().trunc_subsecs(6))
        .await
        .expect("Device credential rotation should succeed");

    let stored = credential_repository
        .find_active_by_device_id(&device.id())
        .await
        .expect("Device credential lookup should succeed")
        .expect("Device credential should exists");

    assert_eq!(stored, new_credential);
    assert_ne!(stored, credential);
}
