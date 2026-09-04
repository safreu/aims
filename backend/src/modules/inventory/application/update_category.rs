use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::{HouseholdEvent, HouseholdId},
            ports::{HouseholdAccessError, HouseholdAccessPolicy, HouseholdEventPublisher},
        },
        inventory::{
            domain::{CategoryId, CategoryName},
            ports::{CategoryRepository, CategoryRepositoryError},
        },
    },
    shared::application::InternalError,
};

pub struct UpdateCategoryCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub category_id: CategoryId,
    pub name: String,
}

pub struct UpdateCategoryService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    category_repository: Arc<dyn CategoryRepository>,
    household_events_publisher: Arc<dyn HouseholdEventPublisher>,
}

impl UpdateCategoryService {
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

    pub async fn execute(&self, command: UpdateCategoryCommand) -> Result<(), UpdateCategoryError> {
        let name =
            CategoryName::parse(&command.name).map_err(|_| UpdateCategoryError::InvalidName)?;

        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let mut existing = self
            .category_repository
            .find_by_id(&command.category_id, &command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    category_name = %command.name,
                    "Failed to check existing category id",
                );
                InternalError::Failed
            })?
            .ok_or(UpdateCategoryError::CategoryNotFound)?;

        let now = Utc::now();

        existing.rename(name, now);

        self.category_repository
            .update(&existing)
            .await
            .map_err(|error| match error {
                CategoryRepositoryError::CategoryAlreadyExists => {
                    UpdateCategoryError::CategoryNotFound
                }
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        category_id = %command.category_id,
                        "Failed to update category"
                    );
                    UpdateCategoryError::Internal(InternalError::Failed)
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
                UpdateCategoryError::Internal(InternalError::Failed)
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
                UpdateCategoryError::Internal(InternalError::Failed)
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
                UpdateCategoryError::Internal(InternalError::Failed)
            })?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum UpdateCategoryError {
    #[error("Category name is invalid")]
    InvalidName,
    #[error("No category was found")]
    CategoryNotFound,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}
