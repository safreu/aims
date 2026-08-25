use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    modules::{
        accounts::api::CurrentUser,
        devices::api::CurrentDevice,
        households::domain::HouseholdId,
        inventory::domain::InventoryItemId,
        scanning::{
            api::dto::{CreateQrActionRequest, CreateQrActionResponse, QrActionReponse},
            application::{
                CreateQrActionCommand, ExecuteQrActionCommand, ListQrActionsCommand,
                RevokeQrActionCommand,
            },
            domain::QrActionId,
        },
    },
    shared::api::{ApiError, AppState},
};

pub async fn create_qr_action(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
    Json(request): Json<CreateQrActionRequest>,
) -> Result<(StatusCode, Json<CreateQrActionResponse>), ApiError> {
    let command = CreateQrActionCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        item_id: InventoryItemId::from_uuid(request.item_id),
        kind: request.kind,
        amount: request.amount,
    };

    let action_id = state
        .scanning
        .create_qr_action
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok((
        StatusCode::CREATED,
        Json(CreateQrActionResponse {
            id: action_id.into_uuid(),
        }),
    ))
}

pub async fn list_qr_actions(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
) -> Result<Json<Vec<QrActionReponse>>, ApiError> {
    let command = ListQrActionsCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
    };

    let actions = state
        .scanning
        .list_qr_actions
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(
        actions.into_iter().map(QrActionReponse::from).collect(),
    ))
}

pub async fn revoke_qr_action(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, qr_action_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let command = RevokeQrActionCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        qr_action_id: QrActionId::from_uuid(qr_action_id),
    };

    state
        .scanning
        .revoke_qr_action
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn execute_qr_action(
    State(state): State<AppState>,
    current_device: CurrentDevice,
    Path(qr_action_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let command = ExecuteQrActionCommand {
        device_id: current_device.device_id(),
        household_id: current_device.household_id(),
        qr_action_id: QrActionId::from_uuid(qr_action_id),
    };

    state
        .scanning
        .execute_qr_action
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
