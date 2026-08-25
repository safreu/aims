use chrono::{DateTime, Utc};

use crate::modules::{
    households::domain::HouseholdId,
    inventory::domain::InventoryItemId,
    scanning::domain::{QrActionId, QrActionKind},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrAction {
    id: QrActionId,
    household_id: HouseholdId,
    item_id: InventoryItemId,
    kind: QrActionKind,
    amount: u32,
    revoked_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl QrAction {
    pub fn new(
        id: QrActionId,
        household_id: HouseholdId,
        item_id: InventoryItemId,
        kind: QrActionKind,
        amount: u32,
        now: DateTime<Utc>,
    ) -> Result<Self, QrActionError> {
        if amount == 0 {
            return Err(QrActionError::InvalidAmount);
        }

        Ok(Self {
            id,
            household_id,
            item_id,
            kind,
            amount,
            revoked_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_persisted(
        id: QrActionId,
        household_id: HouseholdId,
        item_id: InventoryItemId,
        kind: QrActionKind,
        amount: u32,
        revoked_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, QrActionError> {
        if amount == 0 {
            return Err(QrActionError::InvalidAmount);
        }

        Ok(Self {
            id,
            household_id,
            item_id,
            kind,
            amount,
            revoked_at,
            created_at,
            updated_at,
        })
    }

    pub fn id(&self) -> QrActionId {
        self.id
    }

    pub fn household_id(&self) -> HouseholdId {
        self.household_id
    }

    pub fn item_id(&self) -> InventoryItemId {
        self.item_id
    }

    pub fn kind(&self) -> QrActionKind {
        self.kind
    }

    pub fn amount(&self) -> u32 {
        self.amount
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

    pub fn revoke(&mut self, now: DateTime<Utc>) -> Result<(), QrActionError> {
        if self.revoked_at().is_some() {
            return Err(QrActionError::AlreadyRevoked);
        }

        self.revoked_at = Some(now);
        self.updated_at = now;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum QrActionError {
    #[error("QR action amount must be greater then 0")]
    InvalidAmount,
    #[error("QR action is already revoked")]
    AlreadyRevoked,
}
