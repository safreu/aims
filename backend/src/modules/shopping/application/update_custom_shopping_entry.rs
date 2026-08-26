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
            domain::{CustomShoppingEntryError, CustomShoppingEntryId, CustomShoppingEntryTitle},
            ports::CustomShoppingEntryRepository,
        },
    },
    shared::application::InternalError,
};

pub struct UpdateCustomShoppingEntryCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub entry_id: CustomShoppingEntryId,
    pub title: Option<String>,
    pub quantity: Option<u32>,
    pub priority: Option<String>,
    pub note: Option<Option<String>>,
}

pub struct UpdateCustomShoppingEntryService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    custom_entry_repository: Arc<dyn CustomShoppingEntryRepository>,
}

impl UpdateCustomShoppingEntryService {
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
        command: UpdateCustomShoppingEntryCommand,
    ) -> Result<(), UpdateCustomShoppingEntryError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let mut entry = self
            .custom_entry_repository
            .find_by_id_for_household(&command.entry_id, &command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    entry_id = %command.entry_id,
                    "Failed to load custom shopping entry for household"
                );
                UpdateCustomShoppingEntryError::Internal(InternalError::Failed)
            })?
            .ok_or(UpdateCustomShoppingEntryError::EntryNotFound)?;

        let title = command
            .title
            .map(|title| {
                CustomShoppingEntryTitle::parse(&title)
                    .map_err(|_| UpdateCustomShoppingEntryError::InvalidTitle)
            })
            .transpose()?;

        let priority = command
            .priority
            .map(|priority| {
                InventoryPriority::parse(&priority)
                    .map_err(|_| UpdateCustomShoppingEntryError::InvalidPriority)
            })
            .transpose()?;

        let now = Utc::now();

        if let Some(title) = title {
            entry.rename(title, now).map_err(map_domain_error)?;
        }
        if let Some(quantity) = command.quantity {
            entry
                .set_quantity(quantity, now)
                .map_err(map_domain_error)?;
        }
        if let Some(priority) = priority {
            entry
                .set_priority(priority, now)
                .map_err(map_domain_error)?;
        }
        if let Some(note) = command.note {
            entry.set_note(note, now).map_err(map_domain_error)?;
        }

        self.custom_entry_repository
            .update(&entry)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    entry_id = %command.entry_id,
                    "Failed to persist custom shopping entry"
                );
                UpdateCustomShoppingEntryError::Internal(InternalError::Failed)
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum UpdateCustomShoppingEntryError {
    #[error("Shopping entry title is invalid")]
    InvalidTitle,
    #[error("Invalid inventory priority")]
    InvalidPriority,
    #[error("Invalid shopping entry quantity")]
    InvalidQuantity,
    #[error("Note for custom shopping entry is invalid")]
    InvalidNote,
    #[error("Custom shopping entry not found")]
    EntryNotFound,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

fn map_domain_error(error: CustomShoppingEntryError) -> UpdateCustomShoppingEntryError {
    match error {
        CustomShoppingEntryError::InvalidQuantity => {
            UpdateCustomShoppingEntryError::InvalidQuantity
        }
        CustomShoppingEntryError::InvalidNoteLength => UpdateCustomShoppingEntryError::InvalidNote,
        CustomShoppingEntryError::InvalidTimestamps => {
            tracing::error!(
                error = ?error,
                "Invalid timestamp while updating custom shopping entry",
            );
            UpdateCustomShoppingEntryError::Internal(InternalError::Failed)
        }
    }
}

//TODO: Write these tests
/*
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn household_member_can_update_custom_shopping_entry() {}

    #[tokio::test]
    async fn title_can_be_updated() {}

    #[tokio::test]
    async fn quantity_can_be_updated() {}

    #[tokio::test]
    async fn priority_can_be_updated() {}

    #[tokio::test]
    async fn note_can_be_updated() {}

    #[tokio::test]
    async fn note_can_be_cleared() {}

    #[tokio::test]
    async fn no_changes_are_rejected() {}

    #[tokio::test]
    async fn invalid_title_is_rejected() {}

    #[tokio::test]
    async fn zero_quantity_is_rejected() {}

    #[tokio::test]
    async fn invalid_priority_is_rejected() {}

    #[tokio::test]
    async fn too_long_note_is_rejected() {}

    #[tokio::test]
    async fn missing_entry_returns_not_found() {}

    #[tokio::test]
    async fn non_member_cannot_update_custom_shopping_entry() {}

    #[tokio::test]
    async fn repository_failure_returns_internal_error() {}
}
*/
