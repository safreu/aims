use chrono::{DateTime, Utc};

use crate::modules::{
    accounts::domain::UserId,
    devices::domain::{DeviceId, DeviceName},
    inventory::domain::{
        InventoryItemId, InventoryStockEventId, InventoryStockEventKind, InventoryStockEventSource,
    },
};

pub struct InventoryStockHistoryEntry {
    pub id: InventoryStockEventId,
    pub sequence_number: i64,
    pub item_id: InventoryItemId,
    pub kind: InventoryStockEventKind,
    pub source: InventoryStockEventSource,
    pub amount: Option<u32>,
    pub stock_before: u32,
    pub stock_after: u32,
    pub actor: InventoryStockHistoryActor,
    pub created_at: DateTime<Utc>,
}

pub enum InventoryStockHistoryActor {
    User { id: UserId, display_name: String },
    Device { id: DeviceId, name: DeviceName },
    System,
}
