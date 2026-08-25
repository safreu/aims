use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    modules::{
        households::adapters::{DefaultHouseholdAccessPolicy, PostgresHouseholdRepository},
        inventory::adapters::{
            PostgresInventoryItemRepository, PostgresInventoryStockHistoryQuery,
            PostgresInventoryStockRepository,
        },
        scanning::{
            adapters::PostgresQrActionRepository,
            application::{
                CreateQrActionService, ExecuteQrActionService, ListQrActionsService,
                RevokeQrActionService,
            },
        },
    },
    shared::api::ScanningState,
};

pub(super) fn build_scanning_state(pool: &PgPool) -> ScanningState {
    let household_repository = Arc::new(PostgresHouseholdRepository::new(pool.clone()));
    let household_access_policy = Arc::new(DefaultHouseholdAccessPolicy::new(household_repository));
    let inventory_item_repository = Arc::new(PostgresInventoryItemRepository::new(pool.clone()));
    let inventory_stock_repository = Arc::new(PostgresInventoryStockRepository::new(pool.clone()));
    let _inventory_stock_history_query =
        Arc::new(PostgresInventoryStockHistoryQuery::new(pool.clone()));
    let qr_action_repository = Arc::new(PostgresQrActionRepository::new(pool.clone()));

    let create_qr_action_service = Arc::new(CreateQrActionService::new(
        household_access_policy.clone(),
        inventory_item_repository.clone(),
        qr_action_repository.clone(),
    ));

    let list_qr_actions_service = Arc::new(ListQrActionsService::new(
        household_access_policy.clone(),
        qr_action_repository.clone(),
    ));

    let revoke_qr_action_service = Arc::new(RevokeQrActionService::new(
        household_access_policy.clone(),
        qr_action_repository.clone(),
    ));

    let execute_qr_action_service = Arc::new(ExecuteQrActionService::new(
        inventory_stock_repository.clone(),
        qr_action_repository.clone(),
    ));

    ScanningState {
        create_qr_action: create_qr_action_service,
        list_qr_actions: list_qr_actions_service,
        revoke_qr_action: revoke_qr_action_service,
        execute_qr_action: execute_qr_action_service,
    }
}
