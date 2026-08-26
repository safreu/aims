use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
        shopping::{
            domain::CustomShoppingEntryId,
            ports::{CustomShoppingEntryRepository, CustomShoppingEntryRepositoryError},
        },
    },
    shared::application::InternalError,
};

pub struct DeleteCustomShoppingEntryCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub entry_id: CustomShoppingEntryId,
    pub checked: bool,
}

pub struct DeleteCustomShoppingEntryService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    custom_entry_repository: Arc<dyn CustomShoppingEntryRepository>,
}

impl DeleteCustomShoppingEntryService {
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
        command: DeleteCustomShoppingEntryCommand,
    ) -> Result<(), DeleteCustomShoppingEntryError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        self.custom_entry_repository
            .delete(&command.entry_id, &command.household_id)
            .await
            .map_err(|error| match error {
                CustomShoppingEntryRepositoryError::EntryNotFound => {
                    DeleteCustomShoppingEntryError::EntryNotFound
                }
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        entry_id = %command.entry_id,
                        "Failed to delete custom shopping entry"
                    );
                    DeleteCustomShoppingEntryError::Internal(InternalError::Failed)
                }
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DeleteCustomShoppingEntryError {
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
    async fn household_member_can_delete_custom_shopping_entry() {}

    #[tokio::test]
    async fn missing_entry_returns_not_found() {}

    #[tokio::test]
    async fn entry_from_another_household_returns_not_found() {}

    #[tokio::test]
    async fn non_member_cannot_delete_custom_shopping_entry() {}

    #[tokio::test]
    async fn repository_failure_returns_internal_error() {}
}
*/
