use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::{HouseholdEvent, HouseholdId},
            ports::{HouseholdAccessError, HouseholdAccessPolicy, HouseholdEventPublisher},
        },
        inventory::{
            domain::CategoryId,
            ports::{CategoryRepository, CategoryRepositoryError},
        },
    },
    shared::application::InternalError,
};

pub struct DeleteCategoryCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub category_id: CategoryId,
}

pub struct DeleteCategoryService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    category_repository: Arc<dyn CategoryRepository>,
    household_events_publisher: Arc<dyn HouseholdEventPublisher>,
}

impl DeleteCategoryService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        category_repository: Arc<dyn CategoryRepository>,
        household_events_publisher: Arc<dyn HouseholdEventPublisher>,
    ) -> Self {
        Self {
            household_access_policy,
            category_repository,
            household_events_publisher,
        }
    }

    pub async fn execute(&self, command: DeleteCategoryCommand) -> Result<(), DeleteCategoryError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        self.category_repository
            .delete(&command.category_id, &command.household_id)
            .await
            .map_err(|error| match error {
                CategoryRepositoryError::CategoryNotFound => DeleteCategoryError::CategoryNotFound,
                other => {
                    tracing::error!(
                            error = ?other,
                            household_id = %command.household_id,
                            category_id = %command.category_id,
                            "Failed to delete category"
                    );
                    DeleteCategoryError::Internal(InternalError::Failed)
                }
            })?;

        self.household_events_publisher
            .publish(
                command.household_id,
                HouseholdEvent::InventoryCategoriesChanged,
            )
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    category_id = %command.category_id,
                    "Failed to publish category changed event"
                );
                DeleteCategoryError::Internal(InternalError::Failed)
            })?;

        self.household_events_publisher
            .publish(command.household_id, HouseholdEvent::InventoryItemsChanged)
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    category_id = %command.category_id,
                    "Failed to publish inventory items changed event"
                );
                DeleteCategoryError::Internal(InternalError::Failed)
            })?;

        self.household_events_publisher
            .publish(command.household_id, HouseholdEvent::ShoppingListChanged)
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    category_id = %command.category_id,
                    "Failed to publish shopping list changed event"
                );
                DeleteCategoryError::Internal(InternalError::Failed)
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DeleteCategoryError {
    #[error("Category was not found")]
    CategoryNotFound,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

#[cfg(test)]
mod tests {
    use crate::{
        modules::households::domain::HouseholdKind,
        test_helpers::{
            CategoryTestBuilder, build_delete_category_service, insert_owned_household,
        },
    };

    use super::*;

    #[tokio::test]
    async fn household_member_can_delete_category() {
        let (service, category_repository, household_repository) = build_delete_category_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let category = CategoryTestBuilder::new(household.id())
            .name("Food".to_owned())
            .build();

        category_repository
            .insert(&category)
            .await
            .expect("Category insertion should succeed");

        let command = DeleteCategoryCommand {
            category_id: category.id(),
            household_id: household.id(),
            requester_id: owner_id,
        };

        service
            .execute(command)
            .await
            .expect("Category deletion should succeed");

        let stored = category_repository
            .find_by_id(&category.id(), &household.id())
            .await
            .expect("Category lookup should succeed");

        assert!(stored.is_none())
    }

    #[tokio::test]
    async fn non_member_is_forbidden() {
        let (service, category_repository, household_repository) = build_delete_category_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let category = CategoryTestBuilder::new(household.id())
            .name("Food".to_owned())
            .build();

        category_repository
            .insert(&category)
            .await
            .expect("Category insertion should succeed");

        let command = DeleteCategoryCommand {
            category_id: category.id(),
            household_id: household.id(),
            requester_id: UserId::new(),
        };

        let result = service.execute(command).await;

        assert_eq!(
            result,
            Err(DeleteCategoryError::HouseholdAccess(
                HouseholdAccessError::Forbidden
            ))
        )
    }

    #[tokio::test]
    async fn unknown_category_returns_category_not_found() {
        let (service, category_repository, household_repository) = build_delete_category_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let category = CategoryTestBuilder::new(household.id())
            .name("Food".to_owned())
            .build();

        category_repository
            .insert(&category)
            .await
            .expect("Category insertion should succeed");

        let command = DeleteCategoryCommand {
            category_id: CategoryId::new(),
            household_id: household.id(),
            requester_id: owner_id,
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(DeleteCategoryError::CategoryNotFound))
    }
}
