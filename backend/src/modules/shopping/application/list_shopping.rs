use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
        shopping::{ports::ShoppingListQuery, read_models::InventoryShoppingEntry},
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
}

impl ListShoppingService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        shopping_list_query: Arc<dyn ShoppingListQuery>,
    ) -> Self {
        Self {
            household_access_policy,
            shopping_list_query,
        }
    }

    pub async fn execute(
        &self,
        command: ListShoppingCommand,
    ) -> Result<Vec<InventoryShoppingEntry>, ListShoppingError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        self.shopping_list_query
            .list_inventory_entries(&command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to load shopping list"
                );
                ListShoppingError::Internal(InternalError::Failed)
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
    async fn query_failure_returns_internal_error() {}
}
*/
