use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    modules::{
        households::domain::HouseholdId,
        inventory::domain::InventoryItemId,
        scanning::{
            domain::{QrAction, QrActionId, QrActionKind},
            ports::{QrActionRepository, QrActionRepositoryError},
        },
    },
    shared::db::map_sqlx_error,
};

pub struct PostgresQrActionRepository {
    pool: PgPool,
}

impl PostgresQrActionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn map_failed_revocation(
        &self,
        action_id: &QrActionId,
        household_id: &HouseholdId,
    ) -> Result<(), QrActionRepositoryError> {
        let revoked_at = sqlx::query_scalar!(
            r#"
            SELECT revoked_at
            FROM qr_actions
            WHERE id = $1 AND household_id = $2
            "#,
            action_id.into_uuid(),
            household_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        match revoked_at {
            None => Err(QrActionRepositoryError::QrActionNotFound),
            Some(Some(_)) => Err(QrActionRepositoryError::QrActionRevoked),
            Some(None) => Err(QrActionRepositoryError::Persistence(
                crate::shared::db::PersistenceError::Failed,
            )),
        }
    }
}

#[async_trait]
impl QrActionRepository for PostgresQrActionRepository {
    async fn insert(&self, action: &QrAction) -> Result<(), QrActionRepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO qr_actions (
                id,
                household_id,
                item_id,
                kind,
                amount,
                revoked_at,
                created_at,
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            action.id().into_uuid(),
            action.household_id().into_uuid(),
            action.item_id().into_uuid(),
            action.kind().as_str(),
            i64::from(action.amount()),
            action.revoked_at(),
            action.created_at(),
            action.updated_at(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn find_by_id(
        &self,
        action_id: &QrActionId,
    ) -> Result<Option<QrAction>, QrActionRepositoryError> {
        let row = sqlx::query_as!(
            QrActionRow,
            r#"
            SELECT
                id,
                household_id,
                item_id,
                kind,
                amount,
                revoked_at,
                created_at,
                updated_at
            FROM qr_actions
            WHERE id = $1
           "#,
            action_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(QrAction::try_from).transpose()
    }

    async fn find_active_for_household(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<QrAction>, QrActionRepositoryError> {
        let rows = sqlx::query_as!(
            QrActionRow,
            r#"
            SELECT
                id,
                household_id,
                item_id,
                kind,
                amount,
                revoked_at,
                created_at,
                updated_at
            FROM qr_actions
            WHERE household_id = $1 AND revoked_at IS NULL
           "#,
            household_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(QrAction::try_from).collect()
    }

    async fn revoke(&self, action: &QrAction) -> Result<(), QrActionRepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE qr_actions
            SET
                revoked_at = $3,
                updated_at = $4
            WHERE id = $1 AND household_id = $2
           "#,
            action.id().into_uuid(),
            action.household_id().into_uuid(),
            action.revoked_at(),
            action.updated_at(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return self
                .map_failed_revocation(&action.id(), &action.household_id())
                .await;
        }

        Ok(())
    }
}

struct QrActionRow {
    id: Uuid,
    household_id: Uuid,
    item_id: Uuid,
    kind: String,
    amount: i64,
    revoked_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<QrActionRow> for QrAction {
    type Error = QrActionRepositoryError;

    fn try_from(value: QrActionRow) -> Result<Self, Self::Error> {
        let kind = QrActionKind::parse(&value.kind)
            .map_err(|_| QrActionRepositoryError::InvalidStoredData)?;

        let amount =
            u32::try_from(value.amount).map_err(|_| QrActionRepositoryError::InvalidStoredData)?;

        let action = QrAction::from_persisted(
            QrActionId::from_uuid(value.id),
            HouseholdId::from_uuid(value.household_id),
            InventoryItemId::from_uuid(value.item_id),
            kind,
            amount,
            value.revoked_at,
            value.created_at,
            value.updated_at,
        )
        .map_err(|_| QrActionRepositoryError::InvalidStoredData)?;

        Ok(action)
    }
}
