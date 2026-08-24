use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    modules::{
        accounts::api::CurrentUser,
        devices::{
            api::dto::{
                DeviceCredentialResponse, DeviceResponse, RegisterDeviceRequest,
                RegisterDeviceResponse, RenameDeviceRequest,
            },
            application::{
                IssueDeviceCredentialCommand, ListDevicesCommand, RegisterDeviceCommand,
                RenameDeviceCommand, RevokeDeviceCommand, RotateDeviceCredentialCommand,
            },
            domain::DeviceId,
        },
        households::domain::HouseholdId,
    },
    shared::api::{ApiError, AppState},
};

pub async fn register_device(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
    Json(request): Json<RegisterDeviceRequest>,
) -> Result<(StatusCode, Json<RegisterDeviceResponse>), ApiError> {
    let command = RegisterDeviceCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        name: request.name,
        kind: request.kind,
    };

    let device_id = state
        .device
        .register_device
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterDeviceResponse {
            id: device_id.into_uuid(),
        }),
    ))
}

pub async fn list_devices(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
) -> Result<Json<Vec<DeviceResponse>>, ApiError> {
    let command = ListDevicesCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
    };

    let devices = state
        .device
        .list_devices
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    let response = devices.into_iter().map(DeviceResponse::from).collect();

    Ok(Json(response))
}

pub async fn rename_device(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, device_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<RenameDeviceRequest>,
) -> Result<StatusCode, ApiError> {
    let command = RenameDeviceCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        device_id: DeviceId::from_uuid(device_id),
        name: request.name,
    };

    state
        .device
        .rename_device
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn revoke_device(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, device_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let command = RevokeDeviceCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        device_id: DeviceId::from_uuid(device_id),
    };

    state
        .device
        .revoke_device
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn issue_device_credential(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, device_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DeviceCredentialResponse>), ApiError> {
    let command = IssueDeviceCredentialCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        device_id: DeviceId::from_uuid(device_id),
    };

    let token = state
        .device
        .issue_device_credential
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok((
        StatusCode::CREATED,
        Json(DeviceCredentialResponse {
            token: token.into_string(),
        }),
    ))
}

pub async fn rotate_device_credential(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, device_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<DeviceCredentialResponse>, ApiError> {
    let command = RotateDeviceCredentialCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        device_id: DeviceId::from_uuid(device_id),
    };

    let token = state
        .device
        .rotate_device_credential
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(DeviceCredentialResponse {
        token: token.into_string(),
    }))
}
