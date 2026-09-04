#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HouseholdEvent {
    ShoppingListChanged,
    InventoryCategoriesChanged,
    InventoryItemsChanged,
}
