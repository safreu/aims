use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        devices::domain::DeviceId,
        households::domain::HouseholdId,
        inventory::{
            domain::InventoryStockEventSource,
            ports::{
                InventoryStockRepository, InventoryStockRepositoryError, StockMutationContext,
            },
        },
        scanning::{
            domain::{QrActionId, QrActionKind},
            ports::QrActionRepository,
        },
    },
    shared::application::InternalError,
};

pub struct ExecuteQrActionCommand {
    pub device_id: DeviceId,
    pub household_id: HouseholdId,
    pub qr_action_id: QrActionId,
}

pub struct ExecuteQrActionService {
    inventory_stock_repository: Arc<dyn InventoryStockRepository>,
    qr_action_repository: Arc<dyn QrActionRepository>,
}

impl ExecuteQrActionService {
    pub fn new(
        inventory_stock_repository: Arc<dyn InventoryStockRepository>,
        qr_action_repository: Arc<dyn QrActionRepository>,
    ) -> Self {
        Self {
            inventory_stock_repository,
            qr_action_repository,
        }
    }

    pub async fn execute(
        &self,
        command: ExecuteQrActionCommand,
    ) -> Result<(), ExecuteQrActionError> {
        let action = self
            .qr_action_repository
            .find_by_id_for_household(&command.qr_action_id, &command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    qr_action_id = %command.qr_action_id,
                    "Failed to locate QR action for household"
                );
                ExecuteQrActionError::Internal(InternalError::Failed)
            })?
            .ok_or(ExecuteQrActionError::QrActionNotFound)?;

        if action.is_revoked() {
            return Err(ExecuteQrActionError::QrActionRevoked);
        }

        let context = StockMutationContext {
            actor_user_id: None,
            actor_device_id: Some(command.device_id),
            source: InventoryStockEventSource::Qr,
        };

        let now = Utc::now();

        match action.kind() {
            QrActionKind::Increase => {
                self.inventory_stock_repository
                    .increase(
                        &command.household_id,
                        &action.item_id(),
                        action.amount(),
                        &context,
                        now,
                    )
                    .await
                    .map_err(|error| match error {
                        InventoryStockRepositoryError::ItemNotFound => {
                            ExecuteQrActionError::ItemNotFound
                        }
                        InventoryStockRepositoryError::StockOverflow => {
                            ExecuteQrActionError::StockOverflow
                        }
                        InventoryStockRepositoryError::ItemArchived => {
                            ExecuteQrActionError::ItemArchived
                        }
                        other => {
                            tracing::error!(
                                error = ?other,
                                household_id = %command.household_id,
                                qr_action_id = %command.qr_action_id,
                                item_id = %action.item_id(),
                                "Failed to increase stock while executing QR action",
                            );
                            ExecuteQrActionError::Internal(InternalError::Failed)
                        }
                    })?;
            }

            QrActionKind::Decrease => {
                self.inventory_stock_repository
                    .decrease(
                        &command.household_id,
                        &action.item_id(),
                        action.amount(),
                        &context,
                        now,
                    )
                    .await
                    .map_err(|error| match error {
                        InventoryStockRepositoryError::ItemNotFound => {
                            ExecuteQrActionError::ItemNotFound
                        }
                        InventoryStockRepositoryError::InsufficientStock => {
                            ExecuteQrActionError::InsufficientStock
                        }
                        InventoryStockRepositoryError::ItemArchived => {
                            ExecuteQrActionError::ItemArchived
                        }
                        other => {
                            tracing::error!(
                                error = ?other,
                                household_id = %command.household_id,
                                qr_action_id = %command.qr_action_id,
                                item_id = %action.item_id(),
                                "Failed to decrease stock while executing QR action",
                            );
                            ExecuteQrActionError::Internal(InternalError::Failed)
                        }
                    })?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ExecuteQrActionError {
    #[error("QR action is revoked")]
    QrActionRevoked,
    #[error("QR action not found")]
    QrActionNotFound,
    #[error("Inventory item for QR action is archived")]
    ItemArchived,
    #[error("Inventory item for QR action not found")]
    ItemNotFound,
    #[error("Insufficient stock")]
    InsufficientStock,
    #[error("Stock overflow")]
    StockOverflow,
    #[error(transparent)]
    Internal(#[from] InternalError),
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn device_can_execute_increase_qr_action() {}

    #[tokio::test]
    async fn device_can_execute_decrease_qr_action() {}

    #[tokio::test]
    async fn qr_action_from_other_household_is_not_found() {}

    #[tokio::test]
    async fn revoked_qr_action_is_rejected() {}

    #[tokio::test]
    async fn archived_item_is_rejected() {}

    #[tokio::test]
    async fn insufficient_stock_is_rejected() {}

    #[tokio::test]
    async fn stock_overflow_is_rejected() {}

    #[tokio::test]
    async fn qr_execution_records_device_actor_and_qr_source() {}
}
*/
