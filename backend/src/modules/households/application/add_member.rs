use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::{
        accounts::{
            domain::{Email, UserId},
            ports::UserRepository,
        },
        households::{
            domain::{HouseholdEvent, HouseholdId, HouseholdKind, HouseholdMember, HouseholdRole},
            ports::{HouseholdEventPublisher, HouseholdRepository, HouseholdRepositoryError},
        },
    },
    shared::application::InternalError,
};

pub struct AddHouseholdMemberCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
    pub member_email: String,
}

pub struct AddHouseholdMemberService {
    household_repository: Arc<dyn HouseholdRepository>,
    user_repository: Arc<dyn UserRepository>,
    household_events_publisher: Arc<dyn HouseholdEventPublisher>,
}

impl AddHouseholdMemberService {
    pub fn new(
        household_repository: Arc<dyn HouseholdRepository>,
        user_repository: Arc<dyn UserRepository>,
        household_events_publisher: Arc<dyn HouseholdEventPublisher>,
    ) -> Self {
        Self {
            household_repository,
            user_repository,
            household_events_publisher,
        }
    }

    pub async fn execute(
        &self,
        command: AddHouseholdMemberCommand,
    ) -> Result<(), AddHouseholdMemberError> {
        let email = Email::parse(&command.member_email)
            .map_err(|_| AddHouseholdMemberError::InvalidEmail)?;

        let household = self
            .household_repository
            .find_by_id(&command.household_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to load household",
                );
                AddHouseholdMemberError::Internal(InternalError::Failed)
            })?
            .ok_or(AddHouseholdMemberError::HouseholdNotFound)?;

        if household.kind() == HouseholdKind::Personal {
            return Err(AddHouseholdMemberError::PersonalHousehold);
        }

        let requester = self
            .household_repository
            .find_member(&command.household_id, &command.requester_id)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    requester_id = %command.requester_id,
                    "Failed to load requester membership"
                );
                InternalError::Failed
            })?
            .ok_or(AddHouseholdMemberError::Forbidden)?;

        if requester.role() != HouseholdRole::Owner {
            return Err(AddHouseholdMemberError::Forbidden);
        };

        let user = self
            .user_repository
            .find_by_email(&email)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    "failed to load user to add to household"
                );
                InternalError::Failed
            })?
            .ok_or(AddHouseholdMemberError::UserNotFound)?;

        let existing_member = self
            .household_repository
            .find_member(&command.household_id, &user.id())
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    user_id = %user.id(),
                    "Failed to check existing membership"
                );
                InternalError::Failed
            })?;

        if existing_member.is_some() {
            return Err(AddHouseholdMemberError::MemberAlreadyExists);
        }

        let member = HouseholdMember::new(
            command.household_id,
            user.id(),
            HouseholdRole::Member,
            Utc::now(),
        );

        self.household_repository
            .add_member(&member)
            .await
            .map_err(|error| match error {
                HouseholdRepositoryError::MemberAlreadyExists => {
                    AddHouseholdMemberError::MemberAlreadyExists
                }
                other => {
                    tracing::error!(
                        error = ?other,
                        household_id = %command.household_id,
                        user_id = %user.id(),
                        "failed to add household member",
                    );
                    AddHouseholdMemberError::Internal(InternalError::Failed)
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
                AddHouseholdMemberError::Internal(InternalError::Failed)
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum AddHouseholdMemberError {
    #[error("The email is invalid")]
    InvalidEmail,
    #[error("The household was not found")]
    HouseholdNotFound,
    #[error("You do not have the permissions")]
    Forbidden,
    #[error("User can't be added to a personal household")]
    PersonalHousehold,
    #[error("User was not found")]
    UserNotFound,
    #[error("The member already exists")]
    MemberAlreadyExists,
    #[error(transparent)]
    Internal(#[from] InternalError),
}

#[cfg(test)]
mod tests {
    use crate::{
        modules::{
            accounts::adapters::InMemoryUserRepository,
            households::adapters::{BroadcastHouseholdEvents, InMemoryHouseholdRepository},
        },
        test_helpers::{
            DuplicateOnAddHouseholdRepository, FailingHouseholdRepository, FailingUserRepository,
            build_add_member_service, create_owned_household, create_user,
        },
    };

    use super::*;

    #[tokio::test]
    async fn owner_can_add_member_to_shared_household() {
        let (service, household_repository, user_repository) = build_add_member_service();

        let owner = create_user("owner@email.com");
        let member = create_user("member@email.com");

        user_repository
            .insert(&owner)
            .await
            .expect("Owner should be insertable");

        user_repository
            .insert(&member)
            .await
            .expect("Member should be insertable");

        let (household, owner_membership) =
            create_owned_household(owner.id(), HouseholdKind::Shared);

        household_repository
            .create_with_owner(&household, &owner_membership)
            .await
            .expect("Household creation should succeed");

        let command = AddHouseholdMemberCommand {
            requester_id: owner.id(),
            household_id: household.id(),
            member_email: member.email().to_string(),
        };

        service
            .execute(command)
            .await
            .expect("Adding member should succeed");

        let stored_member = household_repository
            .find_member(&household.id(), &member.id())
            .await
            .expect("Membership lookup should succeed")
            .expect("Member should exist");

        assert_eq!(stored_member.user_id(), member.id());
        assert_eq!(stored_member.role(), HouseholdRole::Member);
    }

    #[tokio::test]
    async fn invalid_email_returns_invalid_email() {
        let (service, household_repository, user_repository) = build_add_member_service();

        let owner = create_user("owner@email.com");

        user_repository
            .insert(&owner)
            .await
            .expect("Owner should be insertable");

        let (household, owner_membership) =
            create_owned_household(owner.id(), HouseholdKind::Shared);

        household_repository
            .create_with_owner(&household, &owner_membership)
            .await
            .expect("Household creation should succeed");

        let command = AddHouseholdMemberCommand {
            requester_id: owner.id(),
            household_id: household.id(),
            member_email: "member_email.com".to_owned(),
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(AddHouseholdMemberError::InvalidEmail))
    }

    #[tokio::test]
    async fn unknown_household_returns_household_not_found() {
        let (service, household_repository, user_repository) = build_add_member_service();

        let owner = create_user("owner@email.com");
        let member = create_user("member@email.com");

        user_repository
            .insert(&owner)
            .await
            .expect("Owner should be insertable");

        user_repository
            .insert(&member)
            .await
            .expect("Member should be insertable");

        let (household, owner_membership) =
            create_owned_household(owner.id(), HouseholdKind::Shared);

        household_repository
            .create_with_owner(&household, &owner_membership)
            .await
            .expect("Household creation should succeed");

        let command = AddHouseholdMemberCommand {
            requester_id: owner.id(),
            household_id: HouseholdId::new(),
            member_email: member.email().to_string(),
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(AddHouseholdMemberError::HouseholdNotFound))
    }

    #[tokio::test]
    async fn personal_household_returns_personal_household() {
        let (service, household_repository, user_repository) = build_add_member_service();

        let owner = create_user("owner@email.com");
        let member = create_user("member@email.com");

        user_repository
            .insert(&owner)
            .await
            .expect("Owner should be insertable");

        user_repository
            .insert(&member)
            .await
            .expect("Member should be insertable");

        let (household, owner_membership) =
            create_owned_household(owner.id(), HouseholdKind::Personal);

        household_repository
            .create_with_owner(&household, &owner_membership)
            .await
            .expect("Household creation should succeed");

        let command = AddHouseholdMemberCommand {
            requester_id: owner.id(),
            household_id: household.id(),
            member_email: member.email().to_string(),
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(AddHouseholdMemberError::PersonalHousehold))
    }

    #[tokio::test]
    async fn requester_who_is_not_a_member_returns_forbidden() {
        let (service, household_repository, user_repository) = build_add_member_service();

        let owner = create_user("owner@email.com");
        let member = create_user("member@email.com");
        let another_member = create_user("another_member@email.com");

        user_repository
            .insert(&owner)
            .await
            .expect("Owner should be insertable");

        user_repository
            .insert(&member)
            .await
            .expect("Member should be insertable");

        user_repository
            .insert(&another_member)
            .await
            .expect("Member should be insertable");

        let (household, owner_membership) =
            create_owned_household(owner.id(), HouseholdKind::Shared);

        household_repository
            .create_with_owner(&household, &owner_membership)
            .await
            .expect("Household creation should succeed");

        let command = AddHouseholdMemberCommand {
            requester_id: another_member.id(),
            household_id: household.id(),
            member_email: member.email().to_string(),
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(AddHouseholdMemberError::Forbidden))
    }

    #[tokio::test]
    async fn requester_with_role_member_returns_forbidden() {
        let (service, household_repository, user_repository) = build_add_member_service();

        let owner = create_user("owner@email.com");
        let member = create_user("member@email.com");
        let another_member = create_user("another_member@email.com");

        user_repository
            .insert(&owner)
            .await
            .expect("Owner should be insertable");

        user_repository
            .insert(&member)
            .await
            .expect("Member should be insertable");

        user_repository
            .insert(&another_member)
            .await
            .expect("Member should be insertable");

        let (household, owner_membership) =
            create_owned_household(owner.id(), HouseholdKind::Shared);

        household_repository
            .create_with_owner(&household, &owner_membership)
            .await
            .expect("Household creation should succeed");

        let command = AddHouseholdMemberCommand {
            requester_id: owner.id(),
            household_id: household.id(),
            member_email: member.email().to_string(),
        };

        service
            .execute(command)
            .await
            .expect("Adding member should succeed");

        let command = AddHouseholdMemberCommand {
            requester_id: member.id(),
            household_id: household.id(),
            member_email: another_member.email().to_string(),
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(AddHouseholdMemberError::Forbidden))
    }

    #[tokio::test]
    async fn unknown_target_user_returns_user_not_found() {
        let (service, household_repository, user_repository) = build_add_member_service();

        let owner = create_user("owner@email.com");

        user_repository
            .insert(&owner)
            .await
            .expect("Owner should be insertable");

        let (household, owner_membership) =
            create_owned_household(owner.id(), HouseholdKind::Shared);

        household_repository
            .create_with_owner(&household, &owner_membership)
            .await
            .expect("Household creation should succeed");

        let command = AddHouseholdMemberCommand {
            requester_id: owner.id(),
            household_id: household.id(),
            member_email: "unknown@email.com".to_owned(),
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(AddHouseholdMemberError::UserNotFound))
    }

    #[tokio::test]
    async fn existing_member_returns_member_already_exists() {
        let (service, household_repository, user_repository) = build_add_member_service();

        let owner = create_user("owner@email.com");
        let member = create_user("member@email.com");

        user_repository
            .insert(&owner)
            .await
            .expect("Owner should be insertable");

        user_repository
            .insert(&member)
            .await
            .expect("Member should be insertable");

        let (household, owner_membership) =
            create_owned_household(owner.id(), HouseholdKind::Shared);

        household_repository
            .create_with_owner(&household, &owner_membership)
            .await
            .expect("Household creation should succeed");

        let command = AddHouseholdMemberCommand {
            requester_id: owner.id(),
            household_id: household.id(),
            member_email: member.email().to_string(),
        };

        service
            .execute(command)
            .await
            .expect("Adding member should succeed");

        let command = AddHouseholdMemberCommand {
            requester_id: owner.id(),
            household_id: household.id(),
            member_email: member.email().to_string(),
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(AddHouseholdMemberError::MemberAlreadyExists))
    }

    #[tokio::test]
    async fn household_repository_failure_returns_internal() {
        let household_repository = Arc::new(FailingHouseholdRepository);
        let user_repository = Arc::new(InMemoryUserRepository::new());
        let household_events = Arc::new(BroadcastHouseholdEvents::new(64));
        let household_events_publisher: Arc<dyn HouseholdEventPublisher> = household_events.clone();

        let service = AddHouseholdMemberService::new(
            household_repository,
            user_repository,
            household_events_publisher,
        );

        let command = AddHouseholdMemberCommand {
            requester_id: UserId::new(),
            household_id: HouseholdId::new(),
            member_email: "valid@email.com".to_owned(),
        };

        let result = service.execute(command).await;

        assert_eq!(
            result,
            Err(AddHouseholdMemberError::Internal(InternalError::Failed))
        )
    }

    #[tokio::test]
    async fn user_repository_failure_returns_internal() {
        let household_repository = Arc::new(InMemoryHouseholdRepository::new());
        let user_repository = Arc::new(FailingUserRepository);
        let household_events = Arc::new(BroadcastHouseholdEvents::new(64));
        let household_events_publisher: Arc<dyn HouseholdEventPublisher> = household_events.clone();

        let service = AddHouseholdMemberService::new(
            household_repository.clone(),
            user_repository,
            household_events_publisher,
        );

        let owner_id = UserId::new();

        let (household, owner) = create_owned_household(owner_id, HouseholdKind::Shared);

        household_repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let command = AddHouseholdMemberCommand {
            requester_id: owner_id,
            household_id: household.id(),
            member_email: "valid@email.com".to_owned(),
        };

        let result = service.execute(command).await;

        assert_eq!(
            result,
            Err(AddHouseholdMemberError::Internal(InternalError::Failed))
        )
    }

    #[tokio::test]
    async fn member_already_exists_from_repository_is_preserved() {
        let inner_household_repository = Arc::new(InMemoryHouseholdRepository::new());

        let household_repository = Arc::new(DuplicateOnAddHouseholdRepository {
            inner: inner_household_repository.clone(),
        });

        let household_events = Arc::new(BroadcastHouseholdEvents::new(64));
        let household_events_publisher: Arc<dyn HouseholdEventPublisher> = household_events.clone();

        let user_repository = Arc::new(InMemoryUserRepository::new());

        let service = AddHouseholdMemberService::new(
            household_repository.clone(),
            user_repository.clone(),
            household_events_publisher,
        );

        let owner = create_user("owner@email.com");
        let member = create_user("member@email.com");

        user_repository
            .insert(&owner)
            .await
            .expect("Owner should be insertable");

        user_repository
            .insert(&member)
            .await
            .expect("Member should be insertable");

        let (household, owner_membership) =
            create_owned_household(owner.id(), HouseholdKind::Shared);

        inner_household_repository
            .create_with_owner(&household, &owner_membership)
            .await
            .expect("Household creation should succeed");

        let command = AddHouseholdMemberCommand {
            requester_id: owner.id(),
            household_id: household.id(),
            member_email: member.email().to_string(),
        };

        let result = service.execute(command).await;

        assert_eq!(result, Err(AddHouseholdMemberError::MemberAlreadyExists))
    }
}
