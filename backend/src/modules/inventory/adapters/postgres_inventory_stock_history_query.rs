use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::accounts::domain::UserId;
use crate::modules::devices::domain::{DeviceId, DeviceName};
use crate::modules::households::domain::HouseholdId;
use crate::modules::inventory::domain::{
    InventoryItemId, InventoryStockEventId, InventoryStockEventKind, InventoryStockEventSource,
};
use crate::modules::inventory::ports::{
    InventoryStockHistoryQuery, InventoryStockHistoryQueryError,
};
use crate::modules::inventory::read_models::{
    InventoryStockHistoryActor, InventoryStockHistoryEntry,
};
use crate::shared::db::map_sqlx_error;

pub struct PostgresInventoryStockHistoryQuery {
    pool: PgPool,
}

impl PostgresInventoryStockHistoryQuery {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct InventoryStockHistoryRow {
    id: Uuid,
    sequence_number: i64,
    item_id: Uuid,
    kind: String,
    source: String,
    amount: Option<i64>,
    stock_before: i64,
    stock_after: i64,
    actor_user_id: Option<Uuid>,
    actor_user_display_name: Option<String>,
    actor_device_id: Option<Uuid>,
    actor_device_name: Option<String>,
    created_at: DateTime<Utc>,
}

#[async_trait]
impl InventoryStockHistoryQuery for PostgresInventoryStockHistoryQuery {
    async fn find_for_item(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
    ) -> Result<Vec<InventoryStockHistoryEntry>, InventoryStockHistoryQueryError> {
        let rows = sqlx::query_as!(
            InventoryStockHistoryRow,
            r#"
            SELECT
                e.id,
                e.sequence_number,
                e.item_id,
                e.kind,
                e.source,
                e.amount,
                e.stock_before,
                e.stock_after,
                e.actor_user_id,
                u.display_name AS "actor_user_display_name?",
                e.actor_device_id,
                d.name AS "actor_device_name?",
                e.created_at
            FROM inventory_stock_events e
            LEFT JOIN users u ON u.id = e.actor_user_id
            LEFT JOIN devices d ON d.id = e.actor_device_id
            WHERE e.household_id = $1 AND e.item_id = $2
            ORDER BY e.sequence_number DESC
            "#,
            household_id.as_uuid(),
            item_id.as_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(InventoryStockHistoryEntry::try_from)
            .collect()
    }
}

impl TryFrom<InventoryStockHistoryRow> for InventoryStockHistoryEntry {
    type Error = InventoryStockHistoryQueryError;

    fn try_from(value: InventoryStockHistoryRow) -> Result<Self, Self::Error> {
        let kind = InventoryStockEventKind::parse(&value.kind)
            .map_err(|_| InventoryStockHistoryQueryError::InvalidStoredData)?;

        let source = InventoryStockEventSource::parse(&value.source)
            .map_err(|_| InventoryStockHistoryQueryError::InvalidStoredData)?;

        let amount = value
            .amount
            .map(|amount| {
                u32::try_from(amount)
                    .map_err(|_| InventoryStockHistoryQueryError::InvalidStoredData)
            })
            .transpose()?;

        let stock_before = u32::try_from(value.stock_before)
            .map_err(|_| InventoryStockHistoryQueryError::InvalidStoredData)?;

        let stock_after = u32::try_from(value.stock_after)
            .map_err(|_| InventoryStockHistoryQueryError::InvalidStoredData)?;

        let actor = match (
            value.actor_user_id,
            value.actor_user_display_name,
            value.actor_device_id,
            value.actor_device_name,
        ) {
            (Some(user_id), Some(display_name), None, None) => InventoryStockHistoryActor::User {
                id: UserId::from_uuid(user_id),
                display_name,
            },
            (None, None, Some(device_id), Some(device_name)) => {
                InventoryStockHistoryActor::Device {
                    id: DeviceId::from_uuid(device_id),
                    name: DeviceName::parse(&device_name)
                        .map_err(|_| InventoryStockHistoryQueryError::InvalidStoredData)?,
                }
            }
            (None, None, None, None) => InventoryStockHistoryActor::System,
            _ => return Err(InventoryStockHistoryQueryError::InvalidStoredData),
        };

        Ok(InventoryStockHistoryEntry {
            id: InventoryStockEventId::from_uuid(value.id),
            sequence_number: value.sequence_number,
            item_id: InventoryItemId::from_uuid(value.item_id),
            kind,
            source,
            amount,
            stock_before,
            stock_after,
            actor,
            created_at: value.created_at,
        })
    }
}
