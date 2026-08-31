use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    modules::{
        households::domain::HouseholdId,
        inventory::{
            domain::{
                CategoryId, CategoryName, InventoryItemId, InventoryItemName, InventoryPriority,
                calculate_shopping_quantity,
            },
            ports::{InventoryItemQuery, InventoryItemQueryError},
            read_models::{CategorySummary, InventoryItemListEntry},
        },
    },
    shared::db::map_sqlx_error,
};

pub struct PostgresInventoryItemQuery {
    pool: PgPool,
}

impl PostgresInventoryItemQuery {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InventoryItemQuery for PostgresInventoryItemQuery {
    async fn find_active_for_household(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<InventoryItemListEntry>, InventoryItemQueryError> {
        let rows = sqlx::query_as!(
            InventoryItemListRow,
            r#"
            SELECT
                i.id,
                i.name,
                i.current_stock,
                i.reorder_threshold,
                i.priority,
                i.category_id,
                c.name AS "category_name?"
            FROM inventory_items i
            LEFT JOIN categories c
                ON c.id = i.category_id AND c.household_id = i.household_id
            WHERE i.household_id = $1 AND i.archived_at IS NULL
            ORDER BY LOWER(i.name), i.id
            "#,
            household_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(InventoryItemListEntry::try_from)
            .collect()
    }

    async fn find_archived_for_household(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<InventoryItemListEntry>, InventoryItemQueryError> {
        let rows = sqlx::query_as!(
            InventoryItemListRow,
            r#"
            SELECT
                i.id,
                i.name,
                i.current_stock,
                i.reorder_threshold,
                i.priority,
                i.category_id,
                c.name AS "category_name?"
            FROM inventory_items i
            LEFT JOIN categories c
                ON c.id = i.category_id AND c.household_id = i.household_id
            WHERE i.household_id = $1 AND i.archived_at IS NOT NULL
            ORDER BY LOWER(i.name), i.id
            "#,
            household_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(InventoryItemListEntry::try_from)
            .collect()
    }

    async fn find_active_by_id(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
    ) -> Result<Option<InventoryItemListEntry>, InventoryItemQueryError> {
        let row = sqlx::query_as!(
            InventoryItemListRow,
            r#"
            SELECT
                i.id,
                i.name,
                i.current_stock,
                i.reorder_threshold,
                i.priority,
                i.category_id,
                c.name AS "category_name?"
            FROM inventory_items i
            LEFT JOIN categories c
                ON c.id = i.category_id AND c.household_id = i.household_id
            WHERE i.household_id = $1 AND i.id = $2 AND i.archived_at IS NULL
            "#,
            household_id.into_uuid(),
            item_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(InventoryItemListEntry::try_from).transpose()
    }
}

struct InventoryItemListRow {
    id: Uuid,
    name: String,
    current_stock: i64,
    reorder_threshold: i64,
    priority: String,
    category_id: Option<Uuid>,
    category_name: Option<String>,
}

impl TryFrom<InventoryItemListRow> for InventoryItemListEntry {
    type Error = InventoryItemQueryError;

    fn try_from(row: InventoryItemListRow) -> Result<Self, Self::Error> {
        let category = match (row.category_id, row.category_name) {
            (Some(id), Some(name)) => Some(CategorySummary {
                id: CategoryId::from_uuid(id),
                name: CategoryName::parse(&name)
                    .map_err(|_| InventoryItemQueryError::InvalidStoredData)?,
            }),
            (None, None) => None,

            _ => return Err(InventoryItemQueryError::InvalidStoredData),
        };

        let name = InventoryItemName::parse(&row.name)
            .map_err(|_| InventoryItemQueryError::InvalidStoredData)?;

        let current_stock = u32::try_from(row.current_stock)
            .map_err(|_| InventoryItemQueryError::InvalidStoredData)?;

        let reorder_threshold = u32::try_from(row.reorder_threshold)
            .map_err(|_| InventoryItemQueryError::InvalidStoredData)?;

        let priority = InventoryPriority::parse(&row.priority)
            .map_err(|_| InventoryItemQueryError::InvalidStoredData)?;

        let shopping_quantity = calculate_shopping_quantity(current_stock, reorder_threshold);

        Ok(InventoryItemListEntry {
            id: InventoryItemId::from_uuid(row.id),
            name,
            category,
            current_stock,
            reorder_threshold,
            priority,
            shopping_quantity,
        })
    }
}
