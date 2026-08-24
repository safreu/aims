use crate::shared::api::{AccountsState, DeviceState, HouseholdsState, InventoryItemState};

#[derive(Clone)]
pub struct AppState {
    pub accounts: AccountsState,
    pub households: HouseholdsState,
    pub inventory: InventoryItemState,
    pub device: DeviceState,
}
