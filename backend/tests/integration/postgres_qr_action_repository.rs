use backend::modules::{
    accounts::{adapters::PostgresUserRepository, ports::UserRepository},
    households::{adapters::PostgresHouseholdRepository, domain::HouseholdKind},
    inventory::{adapters::PostgresInventoryItemRepository, ports::InventoryItemRepository},
    scanning::{
        adapters::PostgresQrActionRepository,
        domain::{QrAction, QrActionId, QrActionKind},
        ports::QrActionRepository,
    },
};
use chrono::{SubsecRound, Utc};
use sqlx::PgPool;

use crate::integration::{
    builders::{InventoryItemTestBuilder, UserTestBuilder},
    helpers::insert_owned_household,
};

#[sqlx::test]
async fn qr_action_can_be_inserted_and_loaded(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_repository = PostgresInventoryItemRepository::new(pool.clone());
    let qr_action_repository = PostgresQrActionRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .name("Tofu")
        .build();

    inventory_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let now = Utc::now().trunc_subsecs(6);

    let action = QrAction::new(
        QrActionId::new(),
        household.id(),
        item.id(),
        QrActionKind::Increase,
        1,
        now,
    )
    .expect("QR action should be valid");

    qr_action_repository
        .insert(&action)
        .await
        .expect("QR action insertion should succeed");

    let stored = qr_action_repository
        .find_by_id_for_household(&action.id(), &household.id())
        .await
        .expect("QR action lookup should succeed")
        .expect("QR action should exist");

    assert_eq!(stored, action);
}

#[sqlx::test]
async fn active_qr_actions_for_household_are_returned(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_repository = PostgresInventoryItemRepository::new(pool.clone());
    let qr_action_repository = PostgresQrActionRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .name("Tofu")
        .build();

    inventory_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let now = Utc::now().trunc_subsecs(6);

    let action = QrAction::new(
        QrActionId::new(),
        household.id(),
        item.id(),
        QrActionKind::Increase,
        1,
        now,
    )
    .expect("QR action should be valid");

    qr_action_repository
        .insert(&action)
        .await
        .expect("QR action insertion should succeed");

    let stored = qr_action_repository
        .find_active_for_household(&household.id())
        .await
        .expect("QR action lookup should succeed");

    assert!(stored.contains(&action));
}

#[sqlx::test]
async fn revoked_qr_actions_are_not_returned_as_active(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_repository = PostgresInventoryItemRepository::new(pool.clone());
    let qr_action_repository = PostgresQrActionRepository::new(pool.clone());

    let owner = UserTestBuilder::new().build();

    user_repository
        .insert(&owner)
        .await
        .expect("User insertion should succeed");

    let (household, _) =
        insert_owned_household(&household_repository, owner.id(), HouseholdKind::Shared).await;

    let item = InventoryItemTestBuilder::new(household.id())
        .name("Tofu")
        .build();

    inventory_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let now = Utc::now().trunc_subsecs(6);

    let mut action = QrAction::new(
        QrActionId::new(),
        household.id(),
        item.id(),
        QrActionKind::Increase,
        1,
        now,
    )
    .expect("QR action should be valid");
    action
        .revoke(Utc::now())
        .expect("QR action revocation should succeed");

    qr_action_repository
        .insert(&action)
        .await
        .expect("QR action insertion should succeed");

    let stored = qr_action_repository
        .find_active_for_household(&household.id())
        .await
        .expect("QR action lookup should succeed");

    assert!(!stored.contains(&action));
}

#[sqlx::test]
async fn qr_actions_from_other_households_are_not_returned(pool: PgPool) {
    let user_repository = PostgresUserRepository::new(pool.clone());
    let household_repository = PostgresHouseholdRepository::new(pool.clone());
    let inventory_repository = PostgresInventoryItemRepository::new(pool.clone());
    let qr_action_repository = PostgresQrActionRepository::new(pool.clone());

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
        .name("Tofu")
        .build();

    inventory_repository
        .insert(&item)
        .await
        .expect("Inventory item insertion should succeed");

    let now = Utc::now().trunc_subsecs(6);

    let action = QrAction::new(
        QrActionId::new(),
        household.id(),
        item.id(),
        QrActionKind::Increase,
        1,
        now,
    )
    .expect("QR action should be valid");
    let another_action = QrAction::new(
        QrActionId::new(),
        another_household.id(),
        item.id(),
        QrActionKind::Increase,
        1,
        now,
    )
    .expect("QR action should be valid");

    qr_action_repository
        .insert(&action)
        .await
        .expect("QR action insertion should succeed");

    let stored = qr_action_repository
        .find_active_for_household(&household.id())
        .await
        .expect("QR action lookup should succeed");

    assert!(stored.contains(&action));
    assert!(!stored.contains(&another_action));
}

//TODO: Write these test after there is something to update
/*
#[sqlx::test]
async fn qr_action_can_be_updated(pool: PgPool) {

}

#[sqlx::test]
async fn updating_unknown_qr_action_returns_not_found(pool: PgPool) {}
*/
