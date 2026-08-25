use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
        inventory::{domain::InventoryItemId, ports::InventoryItemRepository},
        scanning::{
            domain::{QrAction, QrActionId, QrActionKind},
            ports::QrActionRepository,
        },
    },
    shared::application::InternalError,
};

pub struct CreateQrActionCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub item_id: InventoryItemId,
    pub kind: String,
    pub amount: u32,
}

pub struct CreateQrActionService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    inventory_item_repository: Arc<dyn InventoryItemRepository>,
    qr_action_repository: Arc<dyn QrActionRepository>,
}

impl CreateQrActionService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        inventory_item_repository: Arc<dyn InventoryItemRepository>,
        qr_action_repository: Arc<dyn QrActionRepository>,
    ) -> Self {
        Self {
            household_access_policy,
            inventory_item_repository,
            qr_action_repository,
        }
    }

    pub async fn execute(
        &self,
        command: CreateQrActionCommand,
    ) -> Result<QrActionId, CreateQrActionError> {
        let kind =
            QrActionKind::parse(&command.kind).map_err(|_| CreateQrActionError::InvalidKind)?;

        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let item = self
            .inventory_item_repository
            .find_by_id(&command.item_id, &command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    item_id = %command.item_id,
                    "Failed to load inventory item"
                );
                CreateQrActionError::Internal(InternalError::Failed)
            })?
            .ok_or(CreateQrActionError::ItemNotFound)?;

        if item.archived_at().is_some() {
            return Err(CreateQrActionError::ItemArchived);
        }

        let qr_action = QrAction::new(
            QrActionId::new(),
            command.household_id,
            command.item_id,
            kind,
            command.amount,
            Utc::now(),
        )
        .map_err(|_| CreateQrActionError::InvalidAmount)?;

        self.qr_action_repository
            .insert(&qr_action)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    item_id = %command.item_id,
                    "Failed to persist QR action"
                );
                CreateQrActionError::Internal(InternalError::Failed)
            })?;

        Ok(qr_action.id())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CreateQrActionError {
    #[error("Inventory item for QR action is archived")]
    ItemArchived,
    #[error("Inventory item for QR action not found")]
    ItemNotFound,
    #[error("Invalid amount for QR action")]
    InvalidAmount,
    #[error("Invalid QR action kind")]
    InvalidKind,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

//TODO: Write these tests
/*
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn household_member_can_create_qr_action() {}

    #[tokio::test]
    async fn invalid_kind_is_rejected() {}

    #[tokio::test]
    async fn zero_amount_is_rejected() {}

    #[tokio::test]
    async fn unknown_item_returns_not_found() {}

    #[tokio::test]
    async fn archived_item_is_rejected() {}

    #[tokio::test]
    async fn non_member_is_forbidden() {}
}
*/
