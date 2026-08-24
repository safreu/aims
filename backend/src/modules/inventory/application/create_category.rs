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
            domain::{Category, CategoryId, CategoryName},
            ports::{CategoryRepository, CategoryRepositoryError},
        },
    },
    shared::application::InternalError,
};

pub struct CreateCategoryCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub name: String,
}

pub struct CreateCategoryService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    category_repository: Arc<dyn CategoryRepository>,
}

impl CreateCategoryService {
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
        command: CreateCategoryCommand,
    ) -> Result<CategoryId, CreateCategoryError> {
        let name =
            CategoryName::parse(&command.name).map_err(|_| CreateCategoryError::InvalidName)?;

        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let existing = self
            .category_repository
            .find_by_name(&command.household_id, &name)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    category_name = %command.name,
                    "Failed to check existing category name",
                );
                InternalError::Failed
            })?;

        if existing.is_some() {
            return Err(CreateCategoryError::CategoryAlreadyExists);
        }

        let now = Utc::now();
        let category_id = CategoryId::new();

        let category = Category::new(category_id, command.household_id, name, now, now);

        self.category_repository
            .insert(&category)
            .await
            .map_err(|error| match error {
                CategoryRepositoryError::CategoryAlreadyExists => {
                    CreateCategoryError::CategoryAlreadyExists
                }
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        category_id = %category_id,
                        "Failed to insert category"
                    );
                    CreateCategoryError::Internal(InternalError::Failed)
                }
            })?;

        Ok(category_id)
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CreateCategoryError {
    #[error("Category name is invalid")]
    InvalidName,
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
        modules::households::domain::HouseholdKind,
        test_helpers::{build_create_category_service, insert_owned_household},
    };

    use super::*;

    #[tokio::test]
    async fn household_member_can_create_category() {
        let (service, category_repository, household_repository) = build_create_category_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let category_id = service
            .execute(CreateCategoryCommand {
                requester_id: owner_id,
                household_id: household.id(),
                name: "Fruit".to_owned(),
            })
            .await
            .expect("Category insertion should succeed");

        let stored = category_repository
            .find_by_id(&category_id, &household.id())
            .await
            .expect("Category lookup should succeed")
            .expect("Category should exist");

        assert_eq!(stored.id(), category_id);
        assert_eq!(stored.household_id(), household.id());
        assert_eq!(stored.name().normalized(), "fruit");
    }

    #[tokio::test]
    async fn invalid_name_is_rejected() {
        let (service, _, household_repository) = build_create_category_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let result = service
            .execute(CreateCategoryCommand {
                requester_id: owner_id,
                household_id: household.id(),
                name: "        ".to_owned(),
            })
            .await;

        assert_eq!(result, Err(CreateCategoryError::InvalidName))
    }

    #[tokio::test]
    async fn non_member_is_forbidden() {
        let (service, _, household_repository) = build_create_category_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let result = service
            .execute(CreateCategoryCommand {
                requester_id: UserId::new(),
                household_id: household.id(),
                name: "Fruit".to_owned(),
            })
            .await;

        assert_eq!(
            result,
            Err(CreateCategoryError::HouseholdAccess(
                HouseholdAccessError::Forbidden
            ))
        )
    }

    #[tokio::test]
    async fn duplicate_category_name_is_rejected() {
        let (service, _, household_repository) = build_create_category_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        service
            .execute(CreateCategoryCommand {
                requester_id: owner_id,
                household_id: household.id(),
                name: "Fruit".to_owned(),
            })
            .await
            .expect("Category insertion should succeed");

        let result = service
            .execute(CreateCategoryCommand {
                requester_id: owner_id,
                household_id: household.id(),
                name: "fruit".to_owned(),
            })
            .await;

        assert_eq!(result, Err(CreateCategoryError::CategoryAlreadyExists))
    }
}
