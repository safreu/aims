use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::{Household, HouseholdId},
            ports::{HouseholdAccessError, HouseholdAccessPolicy, HouseholdRepository},
        },
    },
    shared::application::InternalError,
};

pub struct GetHouseholdCommand {
    pub household_id: HouseholdId,
    pub requester_id: UserId,
}

pub struct GetHouseholdService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    household_repository: Arc<dyn HouseholdRepository>,
}

impl GetHouseholdService {
    pub fn new(
        household_repository: Arc<dyn HouseholdRepository>,
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    ) -> Self {
        Self {
            household_repository,
            household_access_policy,
        }
    }

    pub async fn execute(
        &self,
        command: GetHouseholdCommand,
    ) -> Result<Household, GetHouseholdError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let household = self
            .household_repository
            .find_by_id(&command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    "failed to load household"
                );
                InternalError::Failed
            })?
            .ok_or(GetHouseholdError::HouseholdAccess(
                HouseholdAccessError::HouseholdNotFound,
            ))?;

        Ok(household)
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum GetHouseholdError {
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}

#[cfg(test)]
mod tests {
    use crate::{
        modules::households::{adapters::DefaultHouseholdAccessPolicy, domain::HouseholdKind},
        test_helpers::{
            FailingHouseholdRepository, build_get_household_service, create_owned_household,
            insert_owned_household,
        },
    };

    use super::*;

    #[tokio::test]
    async fn member_can_retrieve_household() {
        let (service, repository) = build_get_household_service();

        let user_id = UserId::new();

        let (household, _) =
            insert_owned_household(&repository, user_id, HouseholdKind::Shared).await;

        let result = service
            .execute(GetHouseholdCommand {
                household_id: household.id(),
                requester_id: user_id,
            })
            .await
            .expect("Household lookup should succeed");

        assert_eq!(result, household)
    }

    #[tokio::test]
    async fn unknown_household_returns_not_found() {
        let (service, _) = build_get_household_service();

        let result = service
            .execute(GetHouseholdCommand {
                household_id: HouseholdId::new(),
                requester_id: UserId::new(),
            })
            .await;

        assert_eq!(
            result,
            Err(GetHouseholdError::HouseholdAccess(
                HouseholdAccessError::HouseholdNotFound
            ))
        )
    }

    #[tokio::test]
    async fn non_member_returns_forbidden() {
        let (service, repository) = build_get_household_service();

        let user_id = UserId::new();

        let (household, owner) = create_owned_household(user_id, HouseholdKind::Shared);

        repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let result = service
            .execute(GetHouseholdCommand {
                household_id: household.id(),
                requester_id: UserId::new(),
            })
            .await;

        assert_eq!(
            result,
            Err(GetHouseholdError::HouseholdAccess(
                HouseholdAccessError::Forbidden
            ))
        )
    }

    #[tokio::test]
    async fn repository_failure_returns_internal_error() {
        let repository = Arc::new(FailingHouseholdRepository);
        let policy = Arc::new(DefaultHouseholdAccessPolicy::new(repository.clone()));
        let service = GetHouseholdService::new(repository, policy);

        let result = service
            .execute(GetHouseholdCommand {
                household_id: HouseholdId::new(),
                requester_id: UserId::new(),
            })
            .await;

        assert_eq!(
            result,
            Err(GetHouseholdError::HouseholdAccess(
                HouseholdAccessError::Internal(InternalError::Failed)
            ))
        )
    }
}
