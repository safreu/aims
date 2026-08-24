use chrono::{DateTime, Utc};

use crate::modules::{
    devices::domain::{DeviceId, DeviceKind, device_name::DeviceName},
    households::domain::HouseholdId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    id: DeviceId,
    household_id: HouseholdId,
    name: DeviceName,
    kind: DeviceKind,
    revoked_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Device {
    pub fn new(
        id: DeviceId,
        household_id: HouseholdId,
        name: DeviceName,
        kind: DeviceKind,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            household_id,
            name,
            kind,
            revoked_at: None,
            created_at,
            updated_at,
        }
    }

    pub fn new_with_revoked_at(
        id: DeviceId,
        household_id: HouseholdId,
        name: DeviceName,
        kind: DeviceKind,
        revoked_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            household_id,
            name,
            kind,
            revoked_at,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> DeviceId {
        self.id
    }

    pub fn household_id(&self) -> HouseholdId {
        self.household_id
    }

    pub fn name(&self) -> &DeviceName {
        &self.name
    }

    pub fn kind(&self) -> DeviceKind {
        self.kind
    }

    pub fn revoked_at(&self) -> Option<DateTime<Utc>> {
        self.revoked_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[allow(unused)]
    fn ensure_active(&self) -> Result<(), DeviceError> {
        if self.revoked_at().is_some() {
            return Err(DeviceError::Revoked);
        }

        Ok(())
    }

    #[allow(unused)]
    pub fn revoke(&mut self, now: DateTime<Utc>) -> Result<(), DeviceError> {
        if self.revoked_at().is_some() {
            return Err(DeviceError::Revoked);
        }

        self.revoked_at = Some(now);
        self.updated_at = now;

        Ok(())
    }

    #[allow(unused)]
    pub fn restore(&mut self, now: DateTime<Utc>) -> Result<(), DeviceError> {
        if self.revoked_at.is_none() {
            return Err(DeviceError::NotRevoked);
        }

        self.revoked_at = None;
        self.updated_at = now;

        Ok(())
    }

    pub fn rename(&mut self, name: DeviceName, now: DateTime<Utc>) -> Result<(), DeviceError> {
        self.ensure_active()?;

        self.name = name;
        self.updated_at = now;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DeviceError {
    #[error("Device is revoked")]
    Revoked,
    #[error("Device is not revoked")]
    NotRevoked,
}
