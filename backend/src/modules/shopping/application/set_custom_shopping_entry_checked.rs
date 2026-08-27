use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::{HouseholdEvent, HouseholdId},
            ports::{HouseholdAccessError, HouseholdAccessPolicy, HouseholdEventPublisher},
        },
        shopping::{domain::CustomShoppingEntryId, ports::CustomShoppingEntryRepository},
    },
    shared::application::InternalError,
};

pub struct SetCustomShoppingEntryCheckedCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub entry_id: CustomShoppingEntryId,
    pub checked: bool,
}

pub struct SetCustomShoppingEntryCheckedService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    custom_entry_repository: Arc<dyn CustomShoppingEntryRepository>,
    household_events_publisher: Arc<dyn HouseholdEventPublisher>,
}

impl SetCustomShoppingEntryCheckedService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        custom_entry_repository: Arc<dyn CustomShoppingEntryRepository>,
        household_events_publisher: Arc<dyn HouseholdEventPublisher>,
    ) -> Self {
        Self {
            household_access_policy,
            custom_entry_repository,
            household_events_publisher,
        }
    }

    pub async fn execute(
        &self,
        command: SetCustomShoppingEntryCheckedCommand,
    ) -> Result<(), SetCustomShoppingEntryCheckedError> {
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
                SetCustomShoppingEntryCheckedError::Internal(InternalError::Failed)
            })?
            .ok_or(SetCustomShoppingEntryCheckedError::EntryNotFound)?;

        let now = Utc::now();

        let result = if command.checked {
            entry.check(now)
        } else {
            entry.uncheck(now)
        };

        result.map_err(|error| {
            tracing::error!(
                error = ?error,
                household_id = %command.household_id,
                entry_id = %command.entry_id,
                "Failed to check custom shopping entry"
            );
            SetCustomShoppingEntryCheckedError::Internal(InternalError::Failed)
        })?;

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
                SetCustomShoppingEntryCheckedError::Internal(InternalError::Failed)
            })?;

        self.household_events_publisher
            .publish(command.household_id, HouseholdEvent::ShoppingListChanged)
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    item_id = %command.entry_id,
                    "Failed to publish shopping list changed event"
                );
                SetCustomShoppingEntryCheckedError::Internal(InternalError::Failed)
            })?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SetCustomShoppingEntryCheckedError {
    #[error("Custom shopping entry not found")]
    EntryNotFound,
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
    async fn custom_shopping_entry_can_be_checked() {}

    #[tokio::test]
    async fn custom_shopping_entry_can_be_unchecked() {}

    #[tokio::test]
    async fn missing_entry_returns_not_found() {}

    #[tokio::test]
    async fn non_member_cannot_change_checked_state() {}

    #[tokio::test]
    async fn repository_failure_returns_internal_error() {}
}
*/
