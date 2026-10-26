use crate::modules::{
    households::domain::HouseholdId,
    inventory::domain::{CategoryId, InventoryItemId},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryEvent {
    ItemCreated {
        household_id: HouseholdId,
        item_id: InventoryItemId,
    },
    ItemUpdated {
        household_id: HouseholdId,
        item_id: InventoryItemId,
    },
    ItemArchived {
        household_id: HouseholdId,
        item_id: InventoryItemId,
    },
    ItemRestored {
        household_id: HouseholdId,
        item_id: InventoryItemId,
    },
    StockChanged {
        household_id: HouseholdId,
        item_id: InventoryItemId,
    },

    CategoryCreated {
        household_id: HouseholdId,
        category_id: CategoryId,
    },
    CategoryUpdated {
        household_id: HouseholdId,
        category_id: CategoryId,
    },
    CategoryDeleted {
        household_id: HouseholdId,
        category_id: CategoryId,
    },
}
