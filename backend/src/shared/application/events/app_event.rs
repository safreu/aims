use crate::modules::{
    households::domain::HouseholdApplicationEvent, inventory::domain::InventoryEvent,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    Inventory(InventoryEvent),
    Household(HouseholdApplicationEvent),
}

impl From<InventoryEvent> for AppEvent {
    fn from(value: InventoryEvent) -> Self {
        Self::Inventory(value)
    }
}

impl From<HouseholdApplicationEvent> for AppEvent {
    fn from(value: HouseholdApplicationEvent) -> Self {
        Self::Household(value)
    }
}
