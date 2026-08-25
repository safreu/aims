use crate::modules::{
    accounts::domain::UserId, devices::domain::DeviceId,
    inventory::domain::InventoryStockEventSource,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockMutationContext {
    pub actor_user_id: Option<UserId>,
    pub actor_device_id: Option<DeviceId>,
    pub source: InventoryStockEventSource,
}
