use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    modules::{
        devices::{
            adapters::PostgresDeviceRepository,
            application::{
                ListDevicesService, RegisterDeviceService, RenameDeviceService, RevokeDeviceService,
            },
        },
        households::adapters::{DefaultHouseholdAccessPolicy, PostgresHouseholdRepository},
    },
    shared::api::DeviceState,
};

pub(super) fn build_device_state(pool: &PgPool) -> DeviceState {
    let household_repository = Arc::new(PostgresHouseholdRepository::new(pool.clone()));
    let household_access_policy = Arc::new(DefaultHouseholdAccessPolicy::new(household_repository));
    let device_repository = Arc::new(PostgresDeviceRepository::new(pool.clone()));

    let register_device_service = Arc::new(RegisterDeviceService::new(
        household_access_policy.clone(),
        device_repository.clone(),
    ));

    let rename_device_service = Arc::new(RenameDeviceService::new(
        household_access_policy.clone(),
        device_repository.clone(),
    ));

    let revoke_device_service = Arc::new(RevokeDeviceService::new(
        household_access_policy.clone(),
        device_repository.clone(),
    ));

    let list_devices_service = Arc::new(ListDevicesService::new(
        household_access_policy.clone(),
        device_repository.clone(),
    ));

    DeviceState {
        register_device: register_device_service,
        rename_device: rename_device_service,
        revoke_device: revoke_device_service,
        list_devices: list_devices_service,
    }
}
