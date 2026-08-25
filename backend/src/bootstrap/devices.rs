use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    modules::{
        devices::{
            adapters::{
                PostgresDeviceCredentialRepository, PostgresDeviceRepository,
                PostgresDeviceRevocationRepository,
            },
            application::{
                AuthenticateDeviceService, IssueDeviceCredentialService, ListDevicesService,
                RegisterDeviceService, RenameDeviceService, RevokeDeviceService,
                RotateDeviceCredentialService,
            },
        },
        households::adapters::{DefaultHouseholdAccessPolicy, PostgresHouseholdRepository},
    },
    shared::{
        api::DeviceState,
        auth::{SecureTokenGenerator, Sha256TokenHasher},
    },
};

pub(super) fn build_device_state(pool: &PgPool) -> DeviceState {
    let household_repository = Arc::new(PostgresHouseholdRepository::new(pool.clone()));
    let household_access_policy = Arc::new(DefaultHouseholdAccessPolicy::new(household_repository));
    let device_repository = Arc::new(PostgresDeviceRepository::new(pool.clone()));
    let device_credential_repository =
        Arc::new(PostgresDeviceCredentialRepository::new(pool.clone()));
    let token_generator = Arc::new(SecureTokenGenerator::new());
    let token_hasher = Arc::new(Sha256TokenHasher::new());
    let device_revocation_repository =
        Arc::new(PostgresDeviceRevocationRepository::new(pool.clone()));

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
        device_revocation_repository.clone(),
    ));

    let list_devices_service = Arc::new(ListDevicesService::new(
        household_access_policy.clone(),
        device_repository.clone(),
    ));

    let issue_device_credential_service = Arc::new(IssueDeviceCredentialService::new(
        household_access_policy.clone(),
        device_repository.clone(),
        device_credential_repository.clone(),
        token_generator.clone(),
        token_hasher.clone(),
    ));

    let rotate_device_credential_service = Arc::new(RotateDeviceCredentialService::new(
        household_access_policy.clone(),
        device_repository.clone(),
        device_credential_repository.clone(),
        token_generator.clone(),
        token_hasher.clone(),
    ));

    let authenticate_device_service = Arc::new(AuthenticateDeviceService::new(
        device_repository.clone(),
        device_credential_repository.clone(),
        token_hasher.clone(),
    ));

    DeviceState {
        register_device: register_device_service,
        rename_device: rename_device_service,
        revoke_device: revoke_device_service,
        list_devices: list_devices_service,
        issue_device_credential: issue_device_credential_service,
        rotate_device_credential: rotate_device_credential_service,
        authenticate_device: authenticate_device_service,
    }
}
