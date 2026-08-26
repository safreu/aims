use crate::shared::api::{
    AccountsState, DeviceState, HouseholdsState, InventoryItemState, ScanningState, ShoppingState,
};

#[derive(Clone)]
pub struct AppState {
    pub accounts: AccountsState,
    pub households: HouseholdsState,
    pub inventory: InventoryItemState,
    pub device: DeviceState,
    pub scanning: ScanningState,
    pub shopping: ShoppingState,
}
