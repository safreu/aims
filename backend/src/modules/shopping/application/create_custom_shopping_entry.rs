use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
        inventory::domain::InventoryPriority,
        shopping::{
            domain::{
                CustomShoppingEntry, CustomShoppingEntryError, CustomShoppingEntryId,
                CustomShoppingEntryTitle,
            },
            ports::CustomShoppingEntryRepository,
        },
    },
    shared::application::InternalError,
};

pub struct CreateCustomShoppingEntryCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub title: String,
    pub quantity: u32,
    pub priority: String,
    pub note: Option<String>,
}

pub struct CreateCustomShoppingEntryService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    custom_entry_repository: Arc<dyn CustomShoppingEntryRepository>,
}

impl CreateCustomShoppingEntryService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        custom_entry_repository: Arc<dyn CustomShoppingEntryRepository>,
    ) -> Self {
        Self {
            household_access_policy,
            custom_entry_repository,
        }
    }

    pub async fn execute(
        &self,
        command: CreateCustomShoppingEntryCommand,
    ) -> Result<CustomShoppingEntryId, CreateCustomShoppingEntryError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let title = CustomShoppingEntryTitle::parse(&command.title)
            .map_err(|_| CreateCustomShoppingEntryError::InvalidTitle)?;

        let priority = InventoryPriority::parse(&command.priority)
            .map_err(|_| CreateCustomShoppingEntryError::InvalidPriority)?;

        let entry = CustomShoppingEntry::new(
            CustomShoppingEntryId::new(),
            command.household_id,
            title,
            command.quantity,
            priority,
            command.note,
            Utc::now(),
        )
        .map_err(|error| match error {
            CustomShoppingEntryError::InvalidNoteLength => {
                CreateCustomShoppingEntryError::InvalidNote
            }
            CustomShoppingEntryError::InvalidQuantity => {
                CreateCustomShoppingEntryError::InvalidQuantity
            }
            CustomShoppingEntryError::InvalidTimestamps => {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to construct custom shopping entry",
                );
                CreateCustomShoppingEntryError::Internal(InternalError::Failed)
            }
        })?;

        self.custom_entry_repository
            .insert(&entry)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    entry_id = %entry.id(),
                    "Failed to persist custom shopping entry"
                );
                CreateCustomShoppingEntryError::Internal(InternalError::Failed)
            })?;

        Ok(entry.id())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CreateCustomShoppingEntryError {
    #[error("Shopping entry title is invalid")]
    InvalidTitle,
    #[error("Invalid inventory priority")]
    InvalidPriority,
    #[error("Invalid shopping entry quantity")]
    InvalidQuantity,
    #[error("Note for custom shopping entry is invalid")]
    InvalidNote,
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
    async fn household_member_can_create_custom_shopping_entry() {}

    #[tokio::test]
    async fn invalid_title_is_rejected() {}

    #[tokio::test]
    async fn zero_quantity_is_rejected() {}

    #[tokio::test]
    async fn invalid_priority_is_rejected() {}

    #[tokio::test]
    async fn too_long_note_is_rejected() {}

    #[tokio::test]
    async fn non_member_cannot_create_custom_shopping_entry() {}

    #[tokio::test]
    async fn repository_failure_returns_internal_error() {}
}
*/
