use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::{HouseholdEvent, HouseholdId, HouseholdKind, HouseholdName, HouseholdRole},
            ports::{HouseholdEventPublisher, HouseholdRepository, HouseholdRepositoryError},
        },
    },
    shared::application::InternalError,
};

pub struct RenameHouseholdCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub name: String,
}

pub struct RenameHouseholdService {
    household_repository: Arc<dyn HouseholdRepository>,
    household_events_publisher: Arc<dyn HouseholdEventPublisher>,
}

impl RenameHouseholdService {
    pub fn new(
        household_repository: Arc<dyn HouseholdRepository>,
        household_events_publisher: Arc<dyn HouseholdEventPublisher>,
    ) -> Self {
        Self {
            household_repository,
            household_events_publisher,
        }
    }

    pub async fn execute(
        &self,
        command: RenameHouseholdCommand,
    ) -> Result<(), RenameHouseholdError> {
        let name =
            HouseholdName::parse(&command.name).map_err(|_| RenameHouseholdError::InvalidName)?;

        let mut household = self
            .household_repository
            .find_by_id(&command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to load household"
                );
                InternalError::Failed
            })?
            .ok_or(RenameHouseholdError::NotFound)?;

        let requester = self
            .household_repository
            .find_member(&command.household_id, &command.requester_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    requester_id = %command.requester_id,
                    "Failed to load requester membership",
                );
                InternalError::Failed
            })?
            .ok_or(RenameHouseholdError::Forbidden)?;

        if household.kind() == HouseholdKind::Shared && requester.role() != HouseholdRole::Owner {
            return Err(RenameHouseholdError::Forbidden);
        }

        household.rename(name, Utc::now());

        self.household_repository
            .update(&household)
            .await
            .map_err(|error| match error {
                HouseholdRepositoryError::HouseholdNotFound => RenameHouseholdError::NotFound,
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        "Failed to update household",
                    );
                    RenameHouseholdError::Internal(InternalError::Failed)
                }
            })?;

        self.household_events_publisher
            .publish(command.household_id, HouseholdEvent::HouseholdChanged)
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to publish household changed event"
                );
                RenameHouseholdError::Internal(InternalError::Failed)
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RenameHouseholdError {
    #[error("Invalid household name")]
    InvalidName,
    #[error("You do not have permission to update this household")]
    Forbidden,
    #[error("Household not found")]
    NotFound,
    #[error(transparent)]
    Internal(#[from] InternalError),
}

#[cfg(test)]
mod tests {

    use crate::{
        modules::households::adapters::{BroadcastHouseholdEvents, InMemoryHouseholdRepository},
        test_helpers::{
            FailingHouseholdRepository, MissingOnUpdateHouseholdRepository,
            build_rename_household_service, create_owned_household, insert_member,
            insert_owned_household,
        },
    };

    use super::*;

    #[tokio::test]
    async fn owner_can_rename_shared_household() {
        let (service, household_repository) = build_rename_household_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let new_name = "New household name".to_string();

        service
            .execute(RenameHouseholdCommand {
                requester_id: owner_id,
                household_id: household.id(),
                name: new_name.clone(),
            })
            .await
            .expect("Household rename should succeed");

        let stored = household_repository
            .find_by_id(&household.id())
            .await
            .expect("Household lookup should succeed")
            .expect("Household should exist");

        assert_eq!(stored.name().as_str(), new_name)
    }

    #[tokio::test]
    async fn owner_can_rename_a_personal_household() {
        let (service, household_repository) = build_rename_household_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Personal).await;

        let new_name = "New household name".to_string();

        service
            .execute(RenameHouseholdCommand {
                requester_id: owner_id,
                household_id: household.id(),
                name: new_name.clone(),
            })
            .await
            .expect("Household rename should succeed");

        let stored = household_repository
            .find_by_id(&household.id())
            .await
            .expect("Household lookup should succeed")
            .expect("Household should exist");

        assert_eq!(stored.name().as_str(), new_name)
    }

    #[tokio::test]
    async fn invalid_name_returns_invalid_name() {
        let (service, household_repository) = build_rename_household_service();

        let owner_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let new_name = "                    ".to_string();

        let result = service
            .execute(RenameHouseholdCommand {
                requester_id: owner_id,
                household_id: household.id(),
                name: new_name.clone(),
            })
            .await;

        assert_eq!(result, Err(RenameHouseholdError::InvalidName))
    }

    #[tokio::test]
    async fn unknown_household_returns_not_found() {
        let (service, _) = build_rename_household_service();

        let new_name = "New household name".to_string();

        let result = service
            .execute(RenameHouseholdCommand {
                requester_id: UserId::new(),
                household_id: HouseholdId::new(),
                name: new_name.clone(),
            })
            .await;

        assert_eq!(result, Err(RenameHouseholdError::NotFound))
    }

    #[tokio::test]
    async fn non_member_returns_forbidden() {
        let (service, household_repository) = build_rename_household_service();

        let owner_id = UserId::new();
        let member_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        let new_name = "New household name".to_string();

        let result = service
            .execute(RenameHouseholdCommand {
                requester_id: member_id,
                household_id: household.id(),
                name: new_name.clone(),
            })
            .await;

        assert_eq!(result, Err(RenameHouseholdError::Forbidden))
    }

    #[tokio::test]
    async fn shared_household_member_cannot_rename_household() {
        let (service, household_repository) = build_rename_household_service();

        let owner_id = UserId::new();
        let member_id = UserId::new();

        let (household, _) =
            insert_owned_household(&household_repository, owner_id, HouseholdKind::Shared).await;

        insert_member(&household_repository, household.id(), member_id).await;

        let new_name = "New household name".to_string();

        let result = service
            .execute(RenameHouseholdCommand {
                requester_id: member_id,
                household_id: household.id(),
                name: new_name.clone(),
            })
            .await;

        assert_eq!(result, Err(RenameHouseholdError::Forbidden))
    }

    #[tokio::test]
    async fn repository_failure_returns_internal() {
        let repository = Arc::new(FailingHouseholdRepository);
        let household_events = Arc::new(BroadcastHouseholdEvents::new(64));
        let household_events_publisher: Arc<dyn HouseholdEventPublisher> = household_events.clone();
        let service = RenameHouseholdService::new(repository, household_events_publisher);

        let result = service
            .execute(RenameHouseholdCommand {
                requester_id: UserId::new(),
                household_id: HouseholdId::new(),
                name: "Valid name".to_owned(),
            })
            .await;

        assert_eq!(
            result,
            Err(RenameHouseholdError::Internal(InternalError::Failed))
        )
    }

    #[tokio::test]
    async fn household_not_found_is_preserved_as_not_found() {
        let inner = Arc::new(InMemoryHouseholdRepository::new());

        let repository = Arc::new(MissingOnUpdateHouseholdRepository {
            inner: inner.clone(),
        });

        let household_events = Arc::new(BroadcastHouseholdEvents::new(64));
        let household_events_publisher: Arc<dyn HouseholdEventPublisher> = household_events.clone();

        let service = RenameHouseholdService::new(repository, household_events_publisher);

        let owner_id = UserId::new();

        let (household, owner) = create_owned_household(owner_id, HouseholdKind::Shared);

        inner
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let result = service
            .execute(RenameHouseholdCommand {
                requester_id: owner_id,
                household_id: household.id(),
                name: "New Household name".to_owned(),
            })
            .await;

        assert_eq!(result, Err(RenameHouseholdError::NotFound))
    }
}
