use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    modules::{
        households::domain::HouseholdId,
        inventory::domain::InventoryPriority,
        shopping::{
            domain::{CustomShoppingEntry, CustomShoppingEntryId, CustomShoppingEntryTitle},
            ports::{CustomShoppingEntryRepository, CustomShoppingEntryRepositoryError},
        },
    },
    shared::db::map_sqlx_error,
};

pub struct PostgresCustomShoppingEntryRepository {
    pool: PgPool,
}

impl PostgresCustomShoppingEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CustomShoppingEntryRepository for PostgresCustomShoppingEntryRepository {
    async fn insert(
        &self,
        entry: &CustomShoppingEntry,
    ) -> Result<(), CustomShoppingEntryRepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO custom_shopping_entries (
                id,
                household_id,
                title,
                quantity,
                priority,
                note,
                checked,
                created_at,
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            entry.id().into_uuid(),
            entry.household_id().into_uuid(),
            entry.title().as_str(),
            i64::from(entry.quantity()),
            entry.priority().as_str(),
            entry.note(),
            entry.checked(),
            entry.created_at(),
            entry.updated_at(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_custom_shopping_sqlx_error)?;

        Ok(())
    }

    async fn find_by_id_for_household(
        &self,
        entry_id: &CustomShoppingEntryId,
        household_id: &HouseholdId,
    ) -> Result<Option<CustomShoppingEntry>, CustomShoppingEntryRepositoryError> {
        let row = sqlx::query_as!(
            CustomShoppingEntryRow,
            r#"
            SELECT
                id,
                household_id,
                title,
                quantity,
                priority,
                note,
                checked,
                created_at,
                updated_at
            FROM custom_shopping_entries
            WHERE id = $1 AND household_id = $2
            "#,
            entry_id.into_uuid(),
            household_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_custom_shopping_sqlx_error)?;

        row.map(CustomShoppingEntry::try_from).transpose()
    }

    async fn find_for_household(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<CustomShoppingEntry>, CustomShoppingEntryRepositoryError> {
        let rows = sqlx::query_as!(
            CustomShoppingEntryRow,
            r#"
            SELECT
                id,
                household_id,
                title,
                quantity,
                priority,
                note,
                checked,
                created_at,
                updated_at
            FROM custom_shopping_entries
            WHERE household_id = $1 
            ORDER BY created_at ASC
            "#,
            household_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_custom_shopping_sqlx_error)?;

        rows.into_iter()
            .map(CustomShoppingEntry::try_from)
            .collect()
    }

    async fn update(
        &self,
        entry: &CustomShoppingEntry,
    ) -> Result<(), CustomShoppingEntryRepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE custom_shopping_entries
            SET
                title = $3,
                quantity = $4,
                priority = $5,
                note = $6,
                checked = $7,
                updated_at = $8
            WHERE id = $1 AND household_id = $2
            "#,
            entry.id().into_uuid(),
            entry.household_id().into_uuid(),
            entry.title().as_str(),
            i64::from(entry.quantity()),
            entry.priority().as_str(),
            entry.note(),
            entry.checked(),
            entry.updated_at(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_custom_shopping_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CustomShoppingEntryRepositoryError::EntryNotFound);
        }

        Ok(())
    }

    async fn delete(
        &self,
        entry_id: &CustomShoppingEntryId,
        household_id: &HouseholdId,
    ) -> Result<(), CustomShoppingEntryRepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM custom_shopping_entries
            WHERE id = $1 AND household_id = $2
            "#,
            entry_id.into_uuid(),
            household_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_custom_shopping_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CustomShoppingEntryRepositoryError::EntryNotFound);
        }

        Ok(())
    }
}

struct CustomShoppingEntryRow {
    id: Uuid,
    household_id: Uuid,
    title: String,
    quantity: i64,
    priority: String,
    note: Option<String>,
    checked: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<CustomShoppingEntryRow> for CustomShoppingEntry {
    type Error = CustomShoppingEntryRepositoryError;

    fn try_from(value: CustomShoppingEntryRow) -> Result<Self, Self::Error> {
        let title = CustomShoppingEntryTitle::parse(&value.title)
            .map_err(|_| CustomShoppingEntryRepositoryError::InvalidStoredData)?;

        let quantity = u32::try_from(value.quantity)
            .map_err(|_| CustomShoppingEntryRepositoryError::InvalidStoredData)?;

        let priority = InventoryPriority::parse(&value.priority)
            .map_err(|_| CustomShoppingEntryRepositoryError::InvalidStoredData)?;

        CustomShoppingEntry::from_persisted(
            CustomShoppingEntryId::from_uuid(value.id),
            HouseholdId::from_uuid(value.household_id),
            title,
            quantity,
            priority,
            value.note,
            value.checked,
            value.created_at,
            value.updated_at,
        )
        .map_err(|_| CustomShoppingEntryRepositoryError::InvalidStoredData)
    }
}

fn map_custom_shopping_sqlx_error(error: sqlx::Error) -> CustomShoppingEntryRepositoryError {
    CustomShoppingEntryRepositoryError::Persistence(map_sqlx_error(error))
}
