use crate::modules::shopping::{domain::CustomShoppingEntry, read_models::InventoryShoppingEntry};

pub struct ShoppingList {
    pub inventory_entries: Vec<InventoryShoppingEntry>,
    pub custom_entries: Vec<CustomShoppingEntry>,
}
