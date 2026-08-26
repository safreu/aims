use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    modules::{
        households::domain::HouseholdId,
        inventory::domain::InventoryItemId,
        shopping::{
            domain::InventoryShoppingState,
            ports::{InventoryShoppingStateRepository, InventoryShoppingStateRepositoryError},
        },
    },
    shared::db::map_sqlx_error,
};

pub struct PostgresInventoryShoppingStateRepository {
    pool: PgPool,
}

impl PostgresInventoryShoppingStateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InventoryShoppingStateRepository for PostgresInventoryShoppingStateRepository {
    async fn find_by_item(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
    ) -> Result<Option<InventoryShoppingState>, InventoryShoppingStateRepositoryError> {
        let row = sqlx::query_as!(
            InventoryShoppingStateRow,
            r#"
            SELECT
                household_id,
                item_id,
                quantity_override,
                note,
                checked,
                dismissed
            FROM inventory_shopping_states
            WHERE household_id = $1 AND item_id = $2
            "#,
            household_id.into_uuid(),
            item_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(InventoryShoppingState::try_from).transpose()
    }

    async fn upsert(
        &self,
        state: &InventoryShoppingState,
    ) -> Result<(), InventoryShoppingStateRepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO inventory_shopping_states(
                household_id,
                item_id,
                quantity_override,
                note,
                checked,
                dismissed,
                created_at,
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (household_id, item_id)
            DO UPDATE SET
                quantity_override = EXCLUDED.quantity_override,
                note = EXCLUDED.note,
                checked = EXCLUDED.checked,
                dismissed = EXCLUDED.dismissed,
                updated_at = NOW()
            "#,
            state.household_id().into_uuid(),
            state.item_id().into_uuid(),
            state.quantity_override().map(i64::from),
            state.note(),
            state.checked(),
            state.dismissed(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn delete(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
    ) -> Result<(), InventoryShoppingStateRepositoryError> {
        sqlx::query!(
            r#"
            DELETE FROM inventory_shopping_states
            WHERE household_id = $1 AND item_id = $2
            "#,
            household_id.into_uuid(),
            item_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}

struct InventoryShoppingStateRow {
    household_id: Uuid,
    item_id: Uuid,
    quantity_override: Option<i64>,
    note: Option<String>,
    checked: bool,
    dismissed: bool,
}

impl TryFrom<InventoryShoppingStateRow> for InventoryShoppingState {
    type Error = InventoryShoppingStateRepositoryError;

    fn try_from(value: InventoryShoppingStateRow) -> Result<Self, Self::Error> {
        let quantity_override = value
            .quantity_override
            .map(u32::try_from)
            .transpose()
            .map_err(|_| InventoryShoppingStateRepositoryError::InvalidStoredData)?;

        InventoryShoppingState::from_persisted(
            HouseholdId::from_uuid(value.household_id),
            InventoryItemId::from_uuid(value.item_id),
            quantity_override,
            value.note,
            value.checked,
            value.dismissed,
        )
        .map_err(|_| InventoryShoppingStateRepositoryError::InvalidStoredData)
    }
}
