use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::HouseholdId,
            ports::{HouseholdAccessError, HouseholdAccessPolicy},
        },
        inventory::{domain::Category, ports::CategoryRepository},
    },
    shared::application::InternalError,
};

pub struct ListCategoriesCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
}

pub struct ListCategoriesService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    category_repository: Arc<dyn CategoryRepository>,
}

impl ListCategoriesService {
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
        command: ListCategoriesCommand,
    ) -> Result<Vec<Category>, ListCategoriesError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let categories = self
            .category_repository
            .find_for_household(&command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to load categories for household"
                );
                InternalError::Failed
            })?;

        Ok(categories)
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ListCategoriesError {
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
            CategoryTestBuilder, build_list_categories_service, insert_owned_household,
        },
    };

    use super::*;

    #[tokio::test]
    async fn household_member_can_list_categories() {
        let (service, category_repository, household_repository) = build_list_categories_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let command = ListCategoriesCommand {
            requester_id: owner_id,
            household_id: household.id(),
        };

        let category = CategoryTestBuilder::new(household.id())
            .name("Food".to_owned())
            .build();
        let another_category = CategoryTestBuilder::new(household.id())
            .name("Drinks".to_owned())
            .build();

        category_repository
            .insert(&category)
            .await
            .expect("Category insertion should succeed");
        category_repository
            .insert(&another_category)
            .await
            .expect("Category insertion should succeed");

        let categories = service
            .execute(command)
            .await
            .expect("Categories lookup should succeed");

        assert_eq!(categories.len(), 2);
        assert!(categories.contains(&category));
        assert!(categories.contains(&another_category))
    }

    #[tokio::test]
    async fn only_categories_for_requested_household_are_returned() {
        let (service, category_repository, household_repository) = build_list_categories_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;
        let (another_household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let command = ListCategoriesCommand {
            requester_id: owner_id,
            household_id: household.id(),
        };

        let category = CategoryTestBuilder::new(household.id())
            .name("Food".to_owned())
            .build();
        let another_category = CategoryTestBuilder::new(household.id())
            .name("Drinks".to_owned())
            .build();
        let category_from_another_household = CategoryTestBuilder::new(another_household.id())
            .name("Drinks".to_owned())
            .build();

        category_repository
            .insert(&category)
            .await
            .expect("Category insertion should succeed");
        category_repository
            .insert(&another_category)
            .await
            .expect("Category insertion should succeed");
        category_repository
            .insert(&category_from_another_household)
            .await
            .expect("Category insertion should succeed");

        let categories = service
            .execute(command)
            .await
            .expect("Categories lookup should succeed");

        assert_eq!(categories.len(), 2);
        assert!(categories.contains(&category));
        assert!(categories.contains(&another_category));
        assert!(!categories.contains(&category_from_another_household))
    }

    #[tokio::test]
    async fn non_member_is_forbidden() {
        let (service, category_repository, household_repository) = build_list_categories_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let command = ListCategoriesCommand {
            requester_id: UserId::new(),
            household_id: household.id(),
        };

        let category = CategoryTestBuilder::new(household.id())
            .name("Food".to_owned())
            .build();
        let another_category = CategoryTestBuilder::new(household.id())
            .name("Drinks".to_owned())
            .build();

        category_repository
            .insert(&category)
            .await
            .expect("Category insertion should succeed");
        category_repository
            .insert(&another_category)
            .await
            .expect("Category insertion should succeed");

        let result = service.execute(command).await;

        assert_eq!(
            result,
            Err(ListCategoriesError::HouseholdAccess(
                HouseholdAccessError::Forbidden
            ))
        )
    }
}
