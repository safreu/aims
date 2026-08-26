use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    modules::{
        accounts::api::CurrentUser,
        households::domain::HouseholdId,
        inventory::domain::InventoryItemId,
        shopping::{
            api::dto::{
                CreateCustomShoppingRequest, CreateCustomShoppingResponse,
                CustomShoppingEntryResponse, InventoryShoppingEntryResponse,
                SetCustomShoppingCheckedRequest, SetShoppingCheckedRequest, SetShoppingNoteRequest,
                SetShoppingQuantityRequest, ShoppingListResponse, UpdateCustomShoppingRequest,
            },
            application::{
                CreateCustomShoppingEntryCommand, DeleteCustomShoppingEntryCommand,
                DismissShoppingItemCommand, ListShoppingCommand, SetCheckedCommand,
                SetCustomShoppingEntryCheckedCommand, SetNoteCommand, SetShoppingQuantityCommand,
                UpdateCustomShoppingEntryCommand,
            },
            domain::CustomShoppingEntryId,
        },
    },
    shared::api::{ApiError, AppState},
};

pub async fn list_shopping(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
) -> Result<Json<ShoppingListResponse>, ApiError> {
    let command = ListShoppingCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
    };

    let list = state
        .shopping
        .list_shopping
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    let response = ShoppingListResponse {
        inventory_entries: list
            .inventory_entries
            .into_iter()
            .map(InventoryShoppingEntryResponse::from)
            .collect(),
        custom_entries: list
            .custom_entries
            .into_iter()
            .map(CustomShoppingEntryResponse::from)
            .collect(),
    };

    Ok(Json(response))
}

pub async fn set_shopping_quantity(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, item_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<SetShoppingQuantityRequest>,
) -> Result<StatusCode, ApiError> {
    let command = SetShoppingQuantityCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        item_id: InventoryItemId::from_uuid(item_id),
        quantity: request.quantity,
    };

    state
        .shopping
        .set_shopping_quantity
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_shopping_note(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, item_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<SetShoppingNoteRequest>,
) -> Result<StatusCode, ApiError> {
    let command = SetNoteCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        item_id: InventoryItemId::from_uuid(item_id),
        note: request.note,
    };

    state
        .shopping
        .set_note
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_shopping_checked(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, item_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<SetShoppingCheckedRequest>,
) -> Result<StatusCode, ApiError> {
    let command = SetCheckedCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        item_id: InventoryItemId::from_uuid(item_id),
        checked: request.checked,
    };

    state
        .shopping
        .set_checked
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn dismiss_shopping_item(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let command = DismissShoppingItemCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        item_id: InventoryItemId::from_uuid(item_id),
    };

    state
        .shopping
        .dismiss_shopping_item
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_custom_shopping_entry(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
    Json(request): Json<CreateCustomShoppingRequest>,
) -> Result<(StatusCode, Json<CreateCustomShoppingResponse>), ApiError> {
    let command = CreateCustomShoppingEntryCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        title: request.title,
        quantity: request.quantity,
        priority: request.priority,
        note: request.note,
    };

    let entry_id = state
        .shopping
        .create_custom_shopping_entry
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok((
        StatusCode::CREATED,
        Json(CreateCustomShoppingResponse {
            id: entry_id.to_string(),
        }),
    ))
}

pub async fn update_custom_shopping_entry(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, entry_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateCustomShoppingRequest>,
) -> Result<StatusCode, ApiError> {
    let command = UpdateCustomShoppingEntryCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        entry_id: CustomShoppingEntryId::from_uuid(entry_id),
        title: request.title,
        quantity: request.quantity,
        priority: request.priority,
        note: request.note,
    };

    state
        .shopping
        .update_custom_shopping_entry
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_custom_shopping_entry_checked(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, entry_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<SetCustomShoppingCheckedRequest>,
) -> Result<StatusCode, ApiError> {
    let command = SetCustomShoppingEntryCheckedCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        entry_id: CustomShoppingEntryId::from_uuid(entry_id),
        checked: request.checked,
    };

    state
        .shopping
        .set_custom_shopping_entry_checked
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_custom_shopping_entry(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, entry_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let command = DeleteCustomShoppingEntryCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        entry_id: CustomShoppingEntryId::from_uuid(entry_id),
    };

    state
        .shopping
        .delete_custom_shopping_entry
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
