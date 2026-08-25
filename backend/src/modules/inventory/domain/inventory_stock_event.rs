use chrono::{DateTime, Utc};

use crate::modules::{
    accounts::domain::UserId,
    devices::domain::DeviceId,
    households::domain::HouseholdId,
    inventory::domain::{
        InventoryItemId, InventoryStockEventId, InventoryStockEventKind, InventoryStockEventSource,
    },
};

#[allow(unused)]
pub struct InventoryStockEvent {
    id: InventoryStockEventId,
    sequence_number: i64,
    household_id: HouseholdId,
    item_id: InventoryItemId,
    actor_user_id: Option<UserId>,
    actor_device_used: Option<DeviceId>,
    kind: InventoryStockEventKind,
    source: InventoryStockEventSource,
    stock_before: u32,
    stock_after: u32,
    created_at: DateTime<Utc>,
}
