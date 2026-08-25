use crate::shared::api::{
    AccountsState, DeviceState, HouseholdsState, InventoryItemState, ScanningState,
};

#[derive(Clone)]
pub struct AppState {
    pub accounts: AccountsState,
    pub households: HouseholdsState,
    pub inventory: InventoryItemState,
    pub device: DeviceState,
    pub scanning: ScanningState,
}
