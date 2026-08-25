use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::{
    modules::{
        accounts::domain::UserId,
        devices::domain::DeviceId,
        households::domain::HouseholdId,
        inventory::{
            domain::{InventoryItemId, InventoryStockEventId, InventoryStockEventKind},
            ports::{
                InventoryStockRepository, InventoryStockRepositoryError, StockMutationContext,
            },
        },
    },
    shared::db::{PersistenceError, map_sqlx_error},
};

pub struct PostgresInventoryStockRepository {
    pool: PgPool,
}

struct StockStateRow {
    current_stock: i64,
    archived_at: Option<DateTime<Utc>>,
}

struct StockMutationRow {
    stock_before: i64,
    stock_after: i64,
}

impl PostgresInventoryStockRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn find_stock_state(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
    ) -> Result<Option<StockStateRow>, InventoryStockRepositoryError> {
        sqlx::query_as!(
            StockStateRow,
            r#"
            SELECT
                current_stock,
                archived_at
            FROM inventory_items
            WHERE id = $1 AND household_id = $2
            "#,
            item_id.as_uuid(),
            household_id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_stock_sqlx_error)
    }

    async fn map_failed_increase(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
        amount: i64,
    ) -> Result<(), InventoryStockRepositoryError> {
        let state = self.find_stock_state(household_id, item_id).await?;

        match state {
            None => Err(InventoryStockRepositoryError::ItemNotFound),
            Some(state) if state.archived_at.is_some() => {
                Err(InventoryStockRepositoryError::ItemArchived)
            }
            Some(state) if state.current_stock > i64::from(u32::MAX) - amount => {
                Err(InventoryStockRepositoryError::StockOverflow)
            }
            Some(_) => {
                tracing::error!(
                    household_id = %household_id,
                    item_id = %item_id,
                    "Stock increase affected no rows unexpectedly",
                );
                Err(InventoryStockRepositoryError::Persistence(
                    PersistenceError::Failed,
                ))
            }
        }
    }

    async fn map_failed_decrease(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
        amount: i64,
    ) -> Result<(), InventoryStockRepositoryError> {
        let state = self.find_stock_state(household_id, item_id).await?;

        match state {
            None => Err(InventoryStockRepositoryError::ItemNotFound),
            Some(state) if state.archived_at.is_some() => {
                Err(InventoryStockRepositoryError::ItemArchived)
            }
            Some(state) if state.current_stock < amount => {
                Err(InventoryStockRepositoryError::InsufficientStock)
            }
            Some(_) => {
                tracing::error!(
                    household_id = %household_id,
                    item_id = %item_id,
                    "Stock decrease affected no rows unexpectedly",
                );
                Err(InventoryStockRepositoryError::Persistence(
                    PersistenceError::Failed,
                ))
            }
        }
    }

    async fn map_missing_or_archived(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
    ) -> Result<(), InventoryStockRepositoryError> {
        let state = self.find_stock_state(household_id, item_id).await?;

        match state {
            None => Err(InventoryStockRepositoryError::ItemNotFound),
            Some(state) if state.archived_at.is_some() => {
                Err(InventoryStockRepositoryError::ItemArchived)
            }
            Some(_) => {
                tracing::error!(
                    household_id = %household_id,
                    item_id = %item_id,
                    "Stock update affected no rows for an active inventory item"
                );
                Err(InventoryStockRepositoryError::Persistence(
                    PersistenceError::Failed,
                ))
            }
        }
    }
}

#[async_trait]
impl InventoryStockRepository for PostgresInventoryStockRepository {
    async fn increase(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
        amount: u32,
        context: &StockMutationContext,
        now: DateTime<Utc>,
    ) -> Result<(), InventoryStockRepositoryError> {
        let amount = i64::from(amount);

        let mut transaction = self.pool.begin().await.map_err(map_stock_sqlx_error)?;

        let row = sqlx::query_as!(
            StockMutationRow,
            r#"
            UPDATE inventory_items
            SET
                current_stock = current_stock + $3,
                updated_at = $4
            WHERE id = $1
                AND household_id = $2
                AND archived_at IS NULL
                AND current_stock <= $5::BIGINT - $3::BIGINT
            RETURNING
                current_stock - $3 AS "stock_before!",
                current_stock AS "stock_after!"
            "#,
            item_id.as_uuid(),
            household_id.as_uuid(),
            amount,
            now,
            i64::from(u32::MAX),
        )
        .fetch_optional(&mut *transaction)
        .await
        .map_err(map_stock_sqlx_error)?;

        let Some(row) = row else {
            drop(transaction);

            return self
                .map_failed_increase(household_id, item_id, amount)
                .await;
        };

        sqlx::query!(
            r#"
            INSERT INTO inventory_stock_events (
                id,
                household_id,
                item_id,
                actor_user_id,
                actor_device_id,
                kind,
                source,
                amount,
                stock_before,
                stock_after,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            InventoryStockEventId::new().into_uuid(),
            household_id.as_uuid(),
            item_id.as_uuid(),
            context.actor_user_id.map(UserId::into_uuid),
            context.actor_device_id.map(DeviceId::into_uuid),
            InventoryStockEventKind::Increase.as_str(),
            context.source.as_str(),
            amount,
            row.stock_before,
            row.stock_after,
            now,
        )
        .execute(&mut *transaction)
        .await
        .map_err(map_stock_sqlx_error)?;

        transaction.commit().await.map_err(map_stock_sqlx_error)?;

        Ok(())
    }

    async fn decrease(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
        amount: u32,
        context: &StockMutationContext,
        now: DateTime<Utc>,
    ) -> Result<(), InventoryStockRepositoryError> {
        let amount = i64::from(amount);

        let mut transaction = self.pool.begin().await.map_err(map_stock_sqlx_error)?;

        let row = sqlx::query_as!(
            StockMutationRow,
            r#"
            UPDATE inventory_items
            SET
                current_stock = current_stock - $3,
                updated_at = $4
            WHERE id = $1
                AND household_id = $2
                AND archived_at IS NULL
                AND current_stock >= $3
            RETURNING
                current_stock + $3 AS "stock_before!",
                current_stock AS "stock_after!"
            "#,
            item_id.as_uuid(),
            household_id.as_uuid(),
            amount,
            now,
        )
        .fetch_optional(&mut *transaction)
        .await
        .map_err(map_stock_sqlx_error)?;

        let Some(row) = row else {
            drop(transaction);

            return self
                .map_failed_decrease(household_id, item_id, amount)
                .await;
        };

        sqlx::query!(
            r#"
            INSERT INTO inventory_stock_events (
                id,
                household_id,
                item_id,
                actor_user_id,
                actor_device_id,
                kind,
                source,
                amount,
                stock_before,
                stock_after,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            InventoryStockEventId::new().into_uuid(),
            household_id.as_uuid(),
            item_id.as_uuid(),
            context.actor_user_id.map(UserId::into_uuid),
            context.actor_device_id.map(DeviceId::into_uuid),
            InventoryStockEventKind::Decrease.as_str(),
            context.source.as_str(),
            amount,
            row.stock_before,
            row.stock_after,
            now,
        )
        .execute(&mut *transaction)
        .await
        .map_err(map_stock_sqlx_error)?;

        transaction.commit().await.map_err(map_stock_sqlx_error)?;

        Ok(())
    }

    async fn set(
        &self,
        household_id: &HouseholdId,
        item_id: &InventoryItemId,
        stock: u32,
        context: &StockMutationContext,
        now: DateTime<Utc>,
    ) -> Result<(), InventoryStockRepositoryError> {
        let stock = i64::from(stock);

        let mut transaction = self.pool.begin().await.map_err(map_stock_sqlx_error)?;

        let row = sqlx::query_as!(
            StockMutationRow,
            r#"
            WITH old AS (
                SELECT id, current_stock
                FROM inventory_items
                WHERE id = $1
                    AND household_id = $2
                    AND archived_at IS NULL
                FOR UPDATE
            )
            UPDATE inventory_items AS i
            SET
                current_stock = $3,
                updated_at = $4
            FROM old
                WHERE i.id = old.id
            RETURNING
                old.current_stock AS "stock_before!",
                i.current_stock AS "stock_after!"
            "#,
            item_id.as_uuid(),
            household_id.as_uuid(),
            i64::from(stock),
            now,
        )
        .fetch_optional(&mut *transaction)
        .await
        .map_err(map_stock_sqlx_error)?;

        let Some(row) = row else {
            drop(transaction);

            return self.map_missing_or_archived(household_id, item_id).await;
        };

        sqlx::query!(
            r#"
            INSERT INTO inventory_stock_events (
                id,
                household_id,
                item_id,
                actor_user_id,
                actor_device_id,
                kind,
                source,
                amount,
                stock_before,
                stock_after,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            InventoryStockEventId::new().into_uuid(),
            household_id.as_uuid(),
            item_id.as_uuid(),
            context.actor_user_id.map(UserId::into_uuid),
            context.actor_device_id.map(DeviceId::into_uuid),
            InventoryStockEventKind::Set.as_str(),
            context.source.as_str(),
            None::<i64>,
            row.stock_before,
            row.stock_after,
            now,
        )
        .execute(&mut *transaction)
        .await
        .map_err(map_stock_sqlx_error)?;

        transaction.commit().await.map_err(map_stock_sqlx_error)?;

        Ok(())
    }
}

fn map_stock_sqlx_error(error: sqlx::Error) -> InventoryStockRepositoryError {
    InventoryStockRepositoryError::Persistence(map_sqlx_error(error))
}
