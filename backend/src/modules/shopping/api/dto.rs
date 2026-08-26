use serde::{Deserialize, Serialize};

use crate::modules::shopping::{
    domain::CustomShoppingEntry,
    read_models::{InventoryShoppingCategory, InventoryShoppingEntry},
};

#[derive(Debug, Serialize)]
pub struct CreateCustomShoppingResponse {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct ShoppingListResponse {
    pub inventory_entries: Vec<InventoryShoppingEntryResponse>,
    pub custom_entries: Vec<CustomShoppingEntryResponse>,
}

#[derive(Debug, Serialize)]
pub struct InventoryShoppingEntryResponse {
    pub item_id: String,
    pub name: String,
    pub category: Option<ShoppingCategoryResponse>,
    pub quantity: u32,
    pub priority: String,
    pub note: Option<String>,
    pub checked: bool,
}

impl From<InventoryShoppingEntry> for InventoryShoppingEntryResponse {
    fn from(value: InventoryShoppingEntry) -> Self {
        Self {
            item_id: value.item_id.to_string(),
            name: value.name,
            category: value.category.map(ShoppingCategoryResponse::from),
            quantity: value.quantity,
            priority: value.priority.to_string(),
            note: value.note,
            checked: value.checked,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ShoppingCategoryResponse {
    pub id: String,
    pub name: String,
}

impl From<InventoryShoppingCategory> for ShoppingCategoryResponse {
    fn from(value: InventoryShoppingCategory) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CustomShoppingEntryResponse {
    pub id: String,
    pub title: String,
    pub quantity: u32,
    pub priority: String,
    pub note: Option<String>,
    pub checked: bool,
}

impl From<CustomShoppingEntry> for CustomShoppingEntryResponse {
    fn from(value: CustomShoppingEntry) -> Self {
        Self {
            id: value.id().to_string(),
            title: value.title().as_str().to_owned(),
            quantity: value.quantity(),
            priority: value.priority().to_string(),
            note: value.note().map(str::to_owned),
            checked: value.checked(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SetShoppingQuantityRequest {
    pub quantity: u32,
}

#[derive(Debug, Deserialize)]
pub struct SetShoppingNoteRequest {
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetShoppingCheckedRequest {
    pub checked: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateCustomShoppingRequest {
    pub title: String,
    pub quantity: u32,
    pub priority: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCustomShoppingRequest {
    pub title: Option<String>,
    pub quantity: Option<u32>,
    pub priority: Option<String>,
    pub note: Option<Option<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SetCustomShoppingCheckedRequest {
    pub checked: bool,
}
