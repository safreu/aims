use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::shared::{
    api::{
        AccountsState, DeviceState, HouseholdsState, InventoryItemState, ScanningState,
        ShoppingState,
    },
    application::events::ApplicationResultDispatcher,
};

#[derive(Clone)]
pub struct AppState {
    pub accounts: AccountsState,
    pub households: HouseholdsState,
    pub inventory: InventoryItemState,
    pub device: DeviceState,
    pub scanning: ScanningState,
    pub shopping: ShoppingState,

    pub application_result_dispatcher: Arc<ApplicationResultDispatcher>,

    pub shutdown: CancellationToken,
}
