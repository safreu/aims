use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::{
                Household, HouseholdApplicationEvent, HouseholdId, HouseholdKind, HouseholdMember,
                HouseholdName, HouseholdRole,
            },
            ports::{HouseholdRepository, HouseholdRepositoryError},
        },
    },
    shared::application::{InternalError, events::ApplicationResult},
};

pub struct CreateHouseholdCommand {
    pub owner_id: UserId,
    pub name: String,
    pub kind: HouseholdKind,
}

pub struct CreateHouseholdService {
    household_repository: Arc<dyn HouseholdRepository>,
}

impl CreateHouseholdService {
    pub fn new(household_repository: Arc<dyn HouseholdRepository>) -> Self {
        Self {
            household_repository,
        }
    }

    pub async fn execute(
        &self,
        command: CreateHouseholdCommand,
    ) -> Result<ApplicationResult<HouseholdId>, CreateHouseholdError> {
        let name =
            HouseholdName::parse(&command.name).map_err(|_| CreateHouseholdError::InvalidName)?;

        let household_id = HouseholdId::new();
        let now = Utc::now();

        let personal_owner_id = match command.kind {
            HouseholdKind::Personal => Some(command.owner_id),
            HouseholdKind::Shared => None,
        };

        let household = Household::new(
            household_id,
            name,
            command.kind,
            personal_owner_id,
            now,
            now,
        )
        .map_err(|error| {
            tracing::error!(
                error = ?error,
                owner_id = %command.owner_id,
                "failed to construct household"
            );
            CreateHouseholdError::Internal(InternalError::Failed)
        })?;

        let owner = HouseholdMember::new(household_id, command.owner_id, HouseholdRole::Owner, now);

        self.household_repository
            .create_with_owner(&household, &owner)
            .await
            .map_err(|error| match error {
                HouseholdRepositoryError::PersonalHouseholdAlreadyExists => {
                    CreateHouseholdError::PersonalHouseholdAlreadyExists
                }
                _ => CreateHouseholdError::Internal(InternalError::Failed),
            })?;

        Ok(ApplicationResult::new(household_id).with_event(
            HouseholdApplicationEvent::MemberAdded {
                household_id,
                member_id: command.owner_id,
            },
        ))
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CreateHouseholdError {
    #[error("Invalid household name")]
    InvalidName,
    #[error("Personal household already exists")]
    PersonalHouseholdAlreadyExists,
    #[error(transparent)]
    Internal(#[from] InternalError),
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        modules::households::domain::HouseholdApplicationEvent,
        shared::application::events::AppEvent,
        test_helpers::{FailingHouseholdRepository, build_create_household_service},
    };

    use super::*;

    fn create_command(owner_id: UserId, kind: HouseholdKind) -> CreateHouseholdCommand {
        CreateHouseholdCommand {
            owner_id,
            name: "this is a valid name".into(),
            kind,
        }
    }

    #[tokio::test]
    async fn personal_household_creation_succeeds() {
        let (service, repository) = build_create_household_service();

        let owner_id = UserId::new();

        let result = service
            .execute(create_command(owner_id, HouseholdKind::Personal))
            .await
            .expect("Household creation should succeed");

        let (household_id, events) = result.into_parts();

        let stored_household = repository
            .find_personal_by_owner(&owner_id)
            .await
            .expect("Household lookup should succeed")
            .expect("Household should exist");

        assert_eq!(household_id, stored_household.id());

        assert_eq!(
            events,
            vec![AppEvent::Household(HouseholdApplicationEvent::Created {
                household_id,
                owner_id,
            })]
        );
    }

    #[tokio::test]
    async fn shared_household_creation_succeeds() {
        let (service, repository) = build_create_household_service();

        let owner_id = UserId::new();

        let result = service
            .execute(create_command(owner_id, HouseholdKind::Shared))
            .await
            .expect("Household creation should succeed");

        let (household_id, events) = result.into_parts();

        let households = repository
            .find_for_user(&owner_id)
            .await
            .expect("Household lookup should succeed");

        assert_eq!(households.len(), 1);
        assert_eq!(household_id, households[0].id());

        assert_eq!(
            events,
            vec![AppEvent::Household(HouseholdApplicationEvent::Created {
                household_id,
                owner_id,
            })]
        );
    }

    #[tokio::test]
    async fn invalid_name_returns_invalid_name() {
        let (service, _) = build_create_household_service();

        let owner_id = UserId::new();

        let result = service
            .execute(CreateHouseholdCommand {
                owner_id,
                name: "       ".into(),
                kind: HouseholdKind::Shared,
            })
            .await;

        assert!(matches!(result, Err(CreateHouseholdError::InvalidName)));
    }

    #[tokio::test]
    async fn second_personal_household_returns_personal_household_already_exists() {
        let (service, _) = build_create_household_service();

        let owner_id = UserId::new();

        service
            .execute(create_command(owner_id, HouseholdKind::Personal))
            .await
            .expect("Household creation should succeed");

        let result = service
            .execute(create_command(owner_id, HouseholdKind::Personal))
            .await;

        assert!(matches!(
            result,
            Err(CreateHouseholdError::PersonalHouseholdAlreadyExists)
        ));
    }

    #[tokio::test]
    async fn repository_failure_returns_internal() {
        let repository = Arc::new(FailingHouseholdRepository);

        let service = CreateHouseholdService::new(repository);

        let owner_id = UserId::new();

        let result = service
            .execute(create_command(owner_id, HouseholdKind::Personal))
            .await;

        assert!(matches!(
            result,
            Err(CreateHouseholdError::Internal(InternalError::Failed))
        ));
    }
}
