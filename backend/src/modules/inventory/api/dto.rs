use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::inventory::read_models::{
    CategorySummary, InventoryItemListEntry, InventoryStockHistoryActor, InventoryStockHistoryEntry,
};

#[derive(Debug, Deserialize)]
pub struct CreateInventoryItemRequest {
    pub category_id: Option<Uuid>,
    pub name: String,
    pub current_stock: u32,
    pub reorder_threshold: u32,
    pub priority: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateInventoryItemResponse {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateCategoryResponse {
    pub id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ListCategoriesResponse {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct InventoryItemResponse {
    pub id: Uuid,
    pub name: String,
    pub category: Option<InventoryItemCategoryResponse>,
    pub current_stock: u32,
    pub reorder_threshold: u32,
    pub priority: String,
    pub shopping_quantity: u32,
}

impl From<InventoryItemListEntry> for InventoryItemResponse {
    fn from(value: InventoryItemListEntry) -> Self {
        Self {
            id: value.id.into_uuid(),
            name: value.name.as_str().to_owned(),
            category: value.category.map(InventoryItemCategoryResponse::from),
            current_stock: value.current_stock,
            reorder_threshold: value.reorder_threshold,
            priority: value.priority.as_str().to_owned(),
            shopping_quantity: value.shopping_quantity,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct InventoryItemCategoryResponse {
    pub id: Uuid,
    pub name: String,
}

impl From<CategorySummary> for InventoryItemCategoryResponse {
    fn from(value: CategorySummary) -> Self {
        Self {
            id: value.id.into_uuid(),
            name: value.name.as_str().to_owned(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateInventoryItemRequest {
    pub name: Option<String>,
    pub category_id: Option<Option<Uuid>>,
    pub reorder_threshold: Option<u32>,
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangeInventoryStockRequest {
    pub amount: u32,
}

#[derive(Debug, Deserialize)]
pub struct SetInventoryStockRequest {
    pub stock: u32,
}

#[derive(Debug, Serialize)]
pub struct InventoryStockHistoryResponse {
    pub id: Uuid,
    pub sequence_number: i64,
    pub item_id: Uuid,
    pub kind: String,
    pub source: String,
    pub amount: Option<u32>,
    pub stock_before: u32,
    pub stock_after: u32,
    pub actor: InventoryStockHistoryActorResponse,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InventoryStockHistoryActorResponse {
    User { id: Uuid, display_name: String },
    Device { id: Uuid, name: String },
    System,
}

impl From<InventoryStockHistoryEntry> for InventoryStockHistoryResponse {
    fn from(value: InventoryStockHistoryEntry) -> Self {
        Self {
            id: value.id.into_uuid(),
            sequence_number: value.sequence_number,
            item_id: value.item_id.into_uuid(),
            kind: value.kind.as_str().to_owned(),
            source: value.source.as_str().to_owned(),
            amount: value.amount,
            stock_before: value.stock_before,
            stock_after: value.stock_after,
            actor: InventoryStockHistoryActorResponse::from(value.actor),
            created_at: value.created_at,
        }
    }
}

impl From<InventoryStockHistoryActor> for InventoryStockHistoryActorResponse {
    fn from(value: InventoryStockHistoryActor) -> Self {
        match value {
            InventoryStockHistoryActor::User { id, display_name } => Self::User {
                id: id.into_uuid(),
                display_name: display_name.as_str().to_owned(),
            },
            InventoryStockHistoryActor::Device { id, name } => Self::Device {
                id: id.into_uuid(),
                name: name.as_str().to_owned(),
            },
            InventoryStockHistoryActor::System => Self::System,
        }
    }
}
