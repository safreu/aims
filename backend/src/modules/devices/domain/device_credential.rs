use chrono::{DateTime, Utc};

use crate::modules::devices::domain::{DeviceCredentialId, DeviceId, DeviceTokenHash};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceCredential {
    id: DeviceCredentialId,
    device_id: DeviceId,
    token_hash: DeviceTokenHash,
    created_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

impl DeviceCredential {
    pub fn new(
        id: DeviceCredentialId,
        device_id: DeviceId,
        token_hash: DeviceTokenHash,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            device_id,
            token_hash,
            created_at,
            revoked_at: None,
        }
    }

    pub fn new_with_revoked_at(
        id: DeviceCredentialId,
        device_id: DeviceId,
        token_hash: DeviceTokenHash,
        created_at: DateTime<Utc>,
        revoked_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            device_id,
            token_hash,
            created_at,
            revoked_at,
        }
    }

    pub fn id(&self) -> DeviceCredentialId {
        self.id
    }

    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub fn token_hash(&self) -> &DeviceTokenHash {
        &self.token_hash
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn revoked_at(&self) -> Option<DateTime<Utc>> {
        self.revoked_at
    }

    pub fn is_revoked(&self) -> bool {
        self.revoked_at().is_some()
    }

    pub fn revoke(&mut self, now: DateTime<Utc>) -> Result<(), DeviceCredentialError> {
        if self.revoked_at().is_some() {
            return Err(DeviceCredentialError::AlreadyRevoked);
        }

        self.revoked_at = Some(now);

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DeviceCredentialError {
    #[error("Device credential is already revoked")]
    AlreadyRevoked,
}
