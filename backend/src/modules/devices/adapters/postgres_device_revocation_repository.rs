use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    modules::{
        devices::{
            domain::{Device, DeviceId},
            ports::{DeviceRevocationRepository, DeviceRevocationRepositoryError},
        },
        households::domain::HouseholdId,
    },
    shared::db::{PersistenceError, map_sqlx_error},
};

pub struct PostgresDeviceRevocationRepository {
    pool: PgPool,
}

impl PostgresDeviceRevocationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn map_failed_revocation(
        &self,
        device_id: &DeviceId,
        household_id: &HouseholdId,
    ) -> Result<(), DeviceRevocationRepositoryError> {
        let revoked_at = sqlx::query_scalar!(
            r#"
            SELECT revoked_at
            FROM devices
            WHERE id = $1 AND household_id = $2
            "#,
            device_id.as_uuid(),
            household_id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_revocation_sqlx_error)?;

        match revoked_at {
            None => Err(DeviceRevocationRepositoryError::DeviceNotFound),
            Some(Some(_)) => Err(DeviceRevocationRepositoryError::DeviceRevoked),
            Some(None) => Err(DeviceRevocationRepositoryError::Persistence(
                PersistenceError::Failed,
            )),
        }
    }
}

#[async_trait]
impl DeviceRevocationRepository for PostgresDeviceRevocationRepository {
    async fn revoke(&self, device: &Device) -> Result<(), DeviceRevocationRepositoryError> {
        let mut transaction = self.pool.begin().await.map_err(map_revocation_sqlx_error)?;

        let result = sqlx::query!(
            r#"
            UPDATE devices
            SET
                revoked_at = $3,
                updated_at = $3
            WHERE id = $1
                AND household_id = $2
                AND revoked_at IS NULL
            "#,
            device.id().into_uuid(),
            device.household_id().into_uuid(),
            device.revoked_at(),
        )
        .execute(&mut *transaction)
        .await
        .map_err(map_revocation_sqlx_error)?;

        if result.rows_affected() == 0 {
            drop(transaction);

            return self
                .map_failed_revocation(&device.id(), &device.household_id())
                .await;
        }

        sqlx::query!(
            r#"
            UPDATE device_credentials
            SET revoked_at = $2
            WHERE device_id = $1 AND revoked_at IS NULL
            "#,
            device.id().into_uuid(),
            device.revoked_at(),
        )
        .execute(&mut *transaction)
        .await
        .map_err(map_revocation_sqlx_error)?;

        transaction
            .commit()
            .await
            .map_err(map_revocation_sqlx_error)?;

        Ok(())
    }
}

fn map_revocation_sqlx_error(error: sqlx::Error) -> DeviceRevocationRepositoryError {
    DeviceRevocationRepositoryError::Persistence(map_sqlx_error(error))
}
