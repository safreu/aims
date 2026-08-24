use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    modules::devices::{
        domain::{DeviceCredential, DeviceCredentialId, DeviceId, DeviceTokenHash},
        ports::{DeviceCredentialRepository, DeviceCredentialRepositoryError},
    },
    shared::db::map_sqlx_error,
};

pub struct PostgresDeviceCredentialRepository {
    pool: PgPool,
}

impl PostgresDeviceCredentialRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeviceCredentialRepository for PostgresDeviceCredentialRepository {
    async fn insert(
        &self,
        credential: &DeviceCredential,
    ) -> Result<(), DeviceCredentialRepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO device_credentials (
            id,
            device_id,
            token_hash,
            created_at,
            revoked_at
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
            credential.id().into_uuid(),
            credential.device_id().into_uuid(),
            credential.token_hash().as_str(),
            credential.created_at(),
            credential.revoked_at(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_insert_error)?;

        Ok(())
    }

    async fn find_active_by_device_id(
        &self,
        device_id: &DeviceId,
    ) -> Result<Option<DeviceCredential>, DeviceCredentialRepositoryError> {
        let row = sqlx::query_as!(
            DeviceCredentialRow,
            r#"
            SELECT
                id,
                device_id,
                token_hash,
                created_at,
                revoked_at
            FROM device_credentials
            WHERE device_id = $1 AND revoked_at IS NULL
            "#,
            device_id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_credential_sqlx_error)?;

        row.map(DeviceCredential::try_from).transpose()
    }

    async fn find_active_by_token_hash(
        &self,
        token_hash: &DeviceTokenHash,
    ) -> Result<Option<DeviceCredential>, DeviceCredentialRepositoryError> {
        let row = sqlx::query_as!(
            DeviceCredentialRow,
            r#"
            SELECT
                id,
                device_id,
                token_hash,
                created_at,
                revoked_at
            FROM device_credentials
            WHERE token_hash = $1 AND revoked_at IS NULL
            "#,
            token_hash.as_str(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_credential_sqlx_error)?;

        row.map(DeviceCredential::try_from).transpose()
    }

    async fn revoke_active(
        &self,
        device_id: &DeviceId,
        now: DateTime<Utc>,
    ) -> Result<(), DeviceCredentialRepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE device_credentials
            SET revoked_at = $2
            WHERE device_id = $1 AND revoked_at IS NULL
            "#,
            device_id.as_uuid(),
            now,
        )
        .execute(&self.pool)
        .await
        .map_err(map_credential_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(DeviceCredentialRepositoryError::CredentialNotFound);
        }

        Ok(())
    }
}

struct DeviceCredentialRow {
    id: Uuid,
    device_id: Uuid,
    token_hash: String,
    created_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

impl TryFrom<DeviceCredentialRow> for DeviceCredential {
    type Error = DeviceCredentialRepositoryError;

    fn try_from(value: DeviceCredentialRow) -> Result<Self, Self::Error> {
        let token_hash = DeviceTokenHash::from_encoded(&value.token_hash)
            .map_err(|_| DeviceCredentialRepositoryError::InvalidStoredData)?;

        Ok(DeviceCredential::new_with_revoked_at(
            DeviceCredentialId::from_uuid(value.id),
            DeviceId::from_uuid(value.device_id),
            token_hash,
            value.created_at,
            value.revoked_at,
        ))
    }
}

fn map_insert_error(error: sqlx::Error) -> DeviceCredentialRepositoryError {
    if let sqlx::Error::Database(database_error) = &error
        && database_error.constraint() == Some("device_credentials_active_device_unique_idx")
    {
        return DeviceCredentialRepositoryError::ActiveCredentialAlreadyExists;
    }

    DeviceCredentialRepositoryError::Persistence(map_sqlx_error(error))
}

fn map_credential_sqlx_error(error: sqlx::Error) -> DeviceCredentialRepositoryError {
    DeviceCredentialRepositoryError::Persistence(map_sqlx_error(error))
}
