use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
        inventory::{
            domain::{CategoryId, CategoryName, InventoryEvent},
            ports::{CategoryRepository, CategoryRepositoryError},
        },
    },
    shared::application::{InternalError, events::ApplicationResult},
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
}

impl UpdateCategoryService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        category_repository: Arc<dyn CategoryRepository>,
    ) -> Self {
        Self {
            household_access_policy,
            category_repository,
        }
    }

    pub async fn execute(
        &self,
        command: UpdateCategoryCommand,
    ) -> Result<ApplicationResult<()>, UpdateCategoryError> {
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
                    UpdateCategoryError::CategoryAlreadyExists
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

        Ok(
            ApplicationResult::new(()).with_event(InventoryEvent::CategoryUpdated {
                household_id: command.household_id,
                category_id: command.category_id,
            }),
        )
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum UpdateCategoryError {
    #[error("Category name is invalid")]
    InvalidName,
    #[error("No category was found")]
    CategoryNotFound,
    #[error("A category with this name already exists")]
    CategoryAlreadyExists,
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

#[cfg(test)]
mod tests {
    use crate::{
        modules::{
            households::{domain::HouseholdKind, ports::HouseholdAccessError},
            inventory::{domain::InventoryEvent, ports::CategoryRepository},
        },
        shared::application::events::AppEvent,
        test_helpers::{
            CategoryTestBuilder, build_update_category_service, insert_owned_household,
        },
    };

    use super::*;

    #[tokio::test]
    async fn household_member_can_update_category() {
        let (service, category_repository, household_repository) = build_update_category_service();

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

        service
            .execute(UpdateCategoryCommand {
                requester_id: owner_id,
                household_id: household.id(),
                category_id: category.id(),
                name: "Fruit".to_owned(),
            })
            .await
            .expect("Category update should succeed");

        let stored = category_repository
            .find_by_id(&category.id(), &household.id())
            .await
            .expect("Category lookup should succeed")
            .expect("Category should exist");

        assert_eq!(stored.name().normalized(), "fruit");
    }

    #[tokio::test]
    async fn updating_category_emits_category_updated_event() {
        let (service, category_repository, household_repository) = build_update_category_service();

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

        let result = service
            .execute(UpdateCategoryCommand {
                requester_id: owner_id,
                household_id: household.id(),
                category_id: category.id(),
                name: "Fruit".to_owned(),
            })
            .await
            .expect("Category update should succeed");

        let (_, events) = result.into_parts();

        assert_eq!(
            events,
            vec![AppEvent::Inventory(InventoryEvent::CategoryUpdated {
                household_id: household.id(),
                category_id: category.id(),
            })]
        );
    }

    #[tokio::test]
    async fn invalid_name_is_rejected() {
        let (service, _, household_repository) = build_update_category_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let result = service
            .execute(UpdateCategoryCommand {
                requester_id: owner_id,
                household_id: household.id(),
                category_id: CategoryId::new(),
                name: "        ".to_owned(),
            })
            .await;

        assert!(matches!(result, Err(UpdateCategoryError::InvalidName)));
    }

    #[tokio::test]
    async fn unknown_category_returns_category_not_found() {
        let (service, _, household_repository) = build_update_category_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let result = service
            .execute(UpdateCategoryCommand {
                requester_id: owner_id,
                household_id: household.id(),
                category_id: CategoryId::new(),
                name: "Fruit".to_owned(),
            })
            .await;

        assert!(matches!(result, Err(UpdateCategoryError::CategoryNotFound)));
    }

    #[tokio::test]
    async fn non_member_is_forbidden() {
        let (service, category_repository, household_repository) = build_update_category_service();

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

        let result = service
            .execute(UpdateCategoryCommand {
                requester_id: UserId::new(),
                household_id: household.id(),
                category_id: category.id(),
                name: "Fruit".to_owned(),
            })
            .await;

        assert!(matches!(
            result,
            Err(UpdateCategoryError::HouseholdAccess(
                HouseholdAccessError::Forbidden
            ))
        ));
    }

    #[tokio::test]
    async fn duplicate_category_name_is_rejected() {
        let (service, category_repository, household_repository) = build_update_category_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let food = CategoryTestBuilder::new(household.id())
            .name("Food".to_owned())
            .build();

        let fruit = CategoryTestBuilder::new(household.id())
            .name("Fruit".to_owned())
            .build();

        category_repository
            .insert(&food)
            .await
            .expect("Category insertion should succeed");

        category_repository
            .insert(&fruit)
            .await
            .expect("Category insertion should succeed");

        let result = service
            .execute(UpdateCategoryCommand {
                requester_id: owner_id,
                household_id: household.id(),
                category_id: food.id(),
                name: "Fruit".to_owned(),
            })
            .await;

        assert!(matches!(
            result,
            Err(UpdateCategoryError::CategoryAlreadyExists)
        ));
    }
}
