use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    modules::{
        households::domain::HouseholdId,
        inventory::domain::{
            CategoryId, InventoryItemId, InventoryPriority, calculate_shopping_quantity,
        },
        shopping::{
            ports::{ShoppingListQuery, ShoppingListQueryError},
            read_models::{InventoryShoppingCategory, InventoryShoppingEntry},
        },
    },
    shared::db::map_sqlx_error,
};

pub struct PostgresShoppingListQuery {
    pool: PgPool,
}

impl PostgresShoppingListQuery {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ShoppingListQuery for PostgresShoppingListQuery {
    async fn list_inventory_entries(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<InventoryShoppingEntry>, ShoppingListQueryError> {
        let rows = sqlx::query_as!(
            InventoryShoppingEntryRow,
            r#"
            SELECT
                i.id AS item_id,
                i.name,
                i.current_stock,
                i.reorder_threshold,
                i.priority,
                c.id AS "category_id?",
                c.name AS "category_name?",
                s.quantity_override AS "quantity_override?",
                s.note AS "note?",
                s.checked AS "checked?",
                s.dismissed AS "dismissed?"
            FROM inventory_items i
            LEFT JOIN categories c
                ON c.id = i.category_id
            LEFT JOIN inventory_shopping_states s
                ON s.item_id = i.id AND s.household_id = i.household_id
            WHERE i.household_id = $1 AND i.archived_at IS NULL
            "#,
            household_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let mut entries = Vec::new();

        for row in rows {
            if let Some(entry) = row.try_into_entry()? {
                entries.push(entry);
            }
        }

        Ok(entries)
    }
}

struct InventoryShoppingEntryRow {
    item_id: Uuid,
    name: String,
    current_stock: i64,
    reorder_threshold: i64,
    priority: String,
    category_id: Option<Uuid>,
    category_name: Option<String>,
    quantity_override: Option<i64>,
    note: Option<String>,
    checked: Option<bool>,
    dismissed: Option<bool>,
}

impl InventoryShoppingEntryRow {
    fn try_into_entry(self) -> Result<Option<InventoryShoppingEntry>, ShoppingListQueryError> {
        let current_stock = u32::try_from(self.current_stock)
            .map_err(|_| ShoppingListQueryError::InvalidStoredData)?;

        let reorder_threshold = u32::try_from(self.reorder_threshold)
            .map_err(|_| ShoppingListQueryError::InvalidStoredData)?;

        let quantity_override = self
            .quantity_override
            .map(u32::try_from)
            .transpose()
            .map_err(|_| ShoppingListQueryError::InvalidStoredData)?;

        let priority = InventoryPriority::parse(&self.priority)
            .map_err(|_| ShoppingListQueryError::InvalidStoredData)?;

        let category = match (self.category_id, self.category_name) {
            (Some(id), Some(name)) => Some(InventoryShoppingCategory {
                id: CategoryId::from_uuid(id),
                name,
            }),
            (None, None) => None,
            _ => return Err(ShoppingListQueryError::InvalidStoredData),
        };

        let calculated_quantity = calculate_shopping_quantity(current_stock, reorder_threshold);

        let quantity = quantity_override.unwrap_or(calculated_quantity);

        let dismissed = self.dismissed.unwrap_or(false);

        if quantity == 0 || dismissed {
            return Ok(None);
        }

        Ok(Some(InventoryShoppingEntry {
            item_id: InventoryItemId::from_uuid(self.item_id),
            name: self.name,
            category,
            quantity,
            priority,
            note: self.note,
            checked: self.checked.unwrap_or(false),
        }))
    }
}
