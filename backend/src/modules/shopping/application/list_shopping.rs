use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
        shopping::{
            ports::{CustomShoppingEntryRepository, ShoppingListQuery},
            read_models::ShoppingList,
        },
    },
    shared::application::InternalError,
};

pub struct ListShoppingCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
}

pub struct ListShoppingService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    shopping_list_query: Arc<dyn ShoppingListQuery>,
    custom_entry_repository: Arc<dyn CustomShoppingEntryRepository>,
}

impl ListShoppingService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        shopping_list_query: Arc<dyn ShoppingListQuery>,
        custom_entry_repository: Arc<dyn CustomShoppingEntryRepository>,
    ) -> Self {
        Self {
            household_access_policy,
            shopping_list_query,
            custom_entry_repository,
        }
    }

    pub async fn execute(
        &self,
        command: ListShoppingCommand,
    ) -> Result<ShoppingList, ListShoppingError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let inventory_entries = self
            .shopping_list_query
            .list_inventory_entries(&command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to load shopping list"
                );
                ListShoppingError::Internal(InternalError::Failed)
            })?;

        let custom_entries = self
            .custom_entry_repository
            .find_for_household(&command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to load custom shopping list entries"
                );
                ListShoppingError::Internal(InternalError::Failed)
            })?;

        Ok(ShoppingList {
            inventory_entries,
            custom_entries,
        })
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ListShoppingError {
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
    async fn household_member_can_list_shopping_entries() {}

    #[tokio::test]
    async fn non_member_cannot_list_shopping_entries() {}

    #[tokio::test]
    async fn household_member_can_list_complete_shopping_list() {}

    #[tokio::test]
    async fn inventory_entries_are_returned() {}

    #[tokio::test]
    async fn custom_entries_are_returned() {}

    #[tokio::test]
    async fn empty_shopping_list_is_returned() {}

    #[tokio::test]
    async fn non_member_cannot_list_shopping_entries() {}

    #[tokio::test]
    async fn inventory_query_failure_returns_internal_error() {}

    #[tokio::test]
    async fn custom_entry_repository_failure_returns_internal_error() {}

    #[tokio::test]
    async fn query_failure_returns_internal_error() {}
}
*/
