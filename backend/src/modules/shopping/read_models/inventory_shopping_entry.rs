use crate::modules::inventory::domain::{CategoryId, InventoryItemId, InventoryPriority};

#[derive(Debug, PartialEq, Eq)]
pub struct InventoryShoppingEntry {
    pub item_id: InventoryItemId,
    pub name: String,
    pub category: Option<InventoryShoppingCategory>,
    pub quantity: u32,
    pub priority: InventoryPriority,
    pub note: Option<String>,
    pub checked: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InventoryShoppingCategory {
    pub id: CategoryId,
    pub name: String,
}
