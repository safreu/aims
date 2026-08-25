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
        inventory::{
            api::dto::{
                ChangeInventoryStockRequest, CreateCategoryRequest, CreateCategoryResponse,
                CreateInventoryItemRequest, CreateInventoryItemResponse, InventoryItemResponse,
                InventoryStockHistoryResponse, ListCategoriesResponse, SetInventoryStockRequest,
                UpdateInventoryItemRequest,
            },
            application::{
                ArchiveInventoryItemCommand, CreateCategoryCommand, CreateInventoryItemCommand,
                DecreaseInventoryStockCommand, DeleteCategoryCommand, GetInventoryItemCommand,
                IncreaseInventoryStockCommand, ListCategoriesCommand, ListInventoryItemsCommand,
                ListInventoryStockHistoryCommand, RestoreInventoryItemCommand,
                SetInventoryStockCommand, UpdateInventoryItemCommand,
            },
            domain::{CategoryId, InventoryItemId, InventoryPriority},
        },
    },
    shared::api::{ApiError, AppState},
};

pub async fn create_inventory_item(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
    Json(request): Json<CreateInventoryItemRequest>,
) -> Result<(StatusCode, Json<CreateInventoryItemResponse>), ApiError> {
    let priority = request
        .priority
        .map(|s| {
            InventoryPriority::parse(&s).map_err(|_| {
                ApiError::bad_request(
                    "invalid_inventory_priority",
                    "The inventory priority is invalid",
                )
            })
        })
        .transpose()?;

    let command = CreateInventoryItemCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        category_id: request.category_id.map(CategoryId::from_uuid),
        name: request.name,
        current_stock: request.current_stock,
        reorder_threshold: request.reorder_threshold,
        priority,
    };

    let item_id = state
        .inventory
        .create_inventory_item
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok((
        StatusCode::CREATED,
        Json(CreateInventoryItemResponse {
            id: item_id.into_uuid(),
        }),
    ))
}

pub async fn create_category(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
    Json(request): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<CreateCategoryResponse>), ApiError> {
    let command = CreateCategoryCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        name: request.name,
    };

    let category_id = state
        .inventory
        .create_category
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok((
        StatusCode::CREATED,
        Json(CreateCategoryResponse {
            id: category_id.into_uuid(),
        }),
    ))
}

pub async fn list_categories(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
) -> Result<Json<Vec<ListCategoriesResponse>>, ApiError> {
    let command = ListCategoriesCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
    };

    let categories = state
        .inventory
        .list_categories
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    let response = categories
        .into_iter()
        .map(|category| ListCategoriesResponse {
            id: category.id().into_uuid(),
            name: category.name().as_str().to_string(),
        })
        .collect();

    Ok(Json(response))
}

pub async fn delete_category(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, category_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let command = DeleteCategoryCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        category_id: CategoryId::from_uuid(category_id),
    };

    state
        .inventory
        .delete_category
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_inventory_items(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
) -> Result<Json<Vec<InventoryItemResponse>>, ApiError> {
    let command = ListInventoryItemsCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
    };

    let items = state
        .inventory
        .list_inventory_items
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(
        items.into_iter().map(InventoryItemResponse::from).collect(),
    ))
}

pub async fn get_inventory_item(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<InventoryItemResponse>, ApiError> {
    let command = GetInventoryItemCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        item_id: InventoryItemId::from_uuid(item_id),
    };

    let item = state
        .inventory
        .get_inventory_item
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(InventoryItemResponse::from(item)))
}

pub async fn update_inventory_item(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, item_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateInventoryItemRequest>,
) -> Result<StatusCode, ApiError> {
    let command = UpdateInventoryItemCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        item_id: InventoryItemId::from_uuid(item_id),
        category_id: request
            .category_id
            .map(|category_id| category_id.map(CategoryId::from_uuid)),
        name: request.name,
        priority: request.priority,
        reorder_threshold: request.reorder_threshold,
    };

    state
        .inventory
        .update_inventory_item
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn archive_inventory_item(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let command = ArchiveInventoryItemCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        item_id: InventoryItemId::from_uuid(item_id),
    };

    state
        .inventory
        .archive_inventory_item
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn restore_inventory_item(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let command = RestoreInventoryItemCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        item_id: InventoryItemId::from_uuid(item_id),
    };

    state
        .inventory
        .restore_inventory_item
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn increase_inventory_stock(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, item_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ChangeInventoryStockRequest>,
) -> Result<StatusCode, ApiError> {
    let command = IncreaseInventoryStockCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        item_id: InventoryItemId::from_uuid(item_id),
        amount: request.amount,
    };

    state
        .inventory
        .increase_inventory_stock
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn decrease_inventory_stock(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, item_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ChangeInventoryStockRequest>,
) -> Result<StatusCode, ApiError> {
    let command = DecreaseInventoryStockCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        item_id: InventoryItemId::from_uuid(item_id),
        amount: request.amount,
    };

    state
        .inventory
        .decrease_inventory_stock
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_inventory_stock(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, item_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<SetInventoryStockRequest>,
) -> Result<StatusCode, ApiError> {
    let command = SetInventoryStockCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        item_id: InventoryItemId::from_uuid(item_id),
        stock: request.stock,
    };

    state
        .inventory
        .set_inventory_stock
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_inventory_stock_history(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<InventoryStockHistoryResponse>>, ApiError> {
    let command = ListInventoryStockHistoryCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        item_id: InventoryItemId::from_uuid(item_id),
    };

    let history = state
        .inventory
        .list_inventory_stock_history
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(
        history
            .into_iter()
            .map(InventoryStockHistoryResponse::from)
            .collect(),
    ))
}
