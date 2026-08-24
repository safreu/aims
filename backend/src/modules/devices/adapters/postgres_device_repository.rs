use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::households::domain::HouseholdId;
use crate::{
    modules::devices::{
        domain::{Device, DeviceId, DeviceKind, DeviceName},
        ports::{DeviceRepository, DeviceRepositoryError},
    },
    shared::db::map_sqlx_error,
};
pub struct PostgresDeviceRepository {
    pool: PgPool,
}

impl PostgresDeviceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeviceRepository for PostgresDeviceRepository {
    async fn insert(&self, device: &Device) -> Result<(), DeviceRepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO devices (
                id,
                household_id,
                name,
                kind,
                revoked_at,
                created_at,
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            device.id().into_uuid(),
            device.household_id().into_uuid(),
            device.name().as_str(),
            device.kind().as_str(),
            device.revoked_at(),
            device.created_at(),
            device.updated_at(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn find_by_id(
        &self,
        device_id: &DeviceId,
        household_id: &HouseholdId,
    ) -> Result<Option<Device>, DeviceRepositoryError> {
        let row = sqlx::query_as!(
            DeviceRow,
            r#"
            SELECT
                id,
                household_id,
                name,
                kind,
                revoked_at,
                created_at,
                updated_at
            FROM devices
            WHERE id = $1 AND household_id = $2
            "#,
            device_id.as_uuid(),
            household_id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Device::try_from).transpose()
    }

    async fn find_active_for_household(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<Device>, DeviceRepositoryError> {
        let rows = sqlx::query_as!(
            DeviceRow,
            r#"
            SELECT
                id,
                household_id,
                name,
                kind,
                revoked_at,
                created_at,
                updated_at
            FROM devices
            WHERE household_id = $1
                AND revoked_at IS NULL
            "#,
            household_id.as_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(Device::try_from).collect()
    }

    async fn update(&self, device: &Device) -> Result<(), DeviceRepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE devices
            SET
                name = $3,
                kind = $4,
                revoked_at = $5,
                updated_at = $6
            WHERE id = $1 AND household_id = $2
            "#,
            device.id().into_uuid(),
            device.household_id().into_uuid(),
            device.name().as_str(),
            device.kind().as_str(),
            device.revoked_at(),
            device.updated_at(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(DeviceRepositoryError::DeviceNotFound);
        }

        Ok(())
    }
}

struct DeviceRow {
    id: Uuid,
    household_id: Uuid,
    name: String,
    kind: String,
    revoked_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<DeviceRow> for Device {
    type Error = DeviceRepositoryError;

    fn try_from(value: DeviceRow) -> Result<Self, Self::Error> {
        let name =
            DeviceName::parse(&value.name).map_err(|_| DeviceRepositoryError::InvalidStoredData)?;

        let kind =
            DeviceKind::parse(&value.kind).map_err(|_| DeviceRepositoryError::InvalidStoredData)?;

        Ok(Device::new_with_revoked_at(
            DeviceId::from_uuid(value.id),
            HouseholdId::from_uuid(value.household_id),
            name,
            kind,
            value.revoked_at,
            value.created_at,
            value.updated_at,
        ))
    }
}
