use crate::{
    modules::{accounts::domain::SessionToken, households::ports::HouseholdRepository},
    shared::auth::{TokenGenerator, TokenGeneratorError},
};

use std::sync::Arc;

use crate::{
    modules::accounts::{
        domain::{Email, PasswordHash, User, UserId},
        ports::{PasswordHasher, PasswordHasherError, UserRepository, UserRepositoryError},
    },
    shared::db::PersistenceError,
};

use async_trait::async_trait;

use crate::modules::households::domain::HouseholdId;
use crate::modules::households::{
    adapters::InMemoryHouseholdRepository,
    domain::{Household, HouseholdMember},
    ports::HouseholdRepositoryError,
};
pub struct FixedSessionTokenGenerator {
    token: String,
}

impl FixedSessionTokenGenerator {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_owned(),
        }
    }
}

impl TokenGenerator<SessionToken> for FixedSessionTokenGenerator {
    fn generate(&self) -> Result<SessionToken, TokenGeneratorError> {
        Ok(SessionToken::from_string(self.token.clone())
            .expect("Test session token should be valid"))
    }
}

pub struct FailingSessionTokenGenerator;

impl TokenGenerator<SessionToken> for FailingSessionTokenGenerator {
    fn generate(&self) -> Result<SessionToken, TokenGeneratorError> {
        Err(TokenGeneratorError::GenerationFailed)
    }
}

pub struct FailingPasswordHasher;

impl PasswordHasher for FailingPasswordHasher {
    #[allow(unused_variables)]
    fn hash(&self, password: &str) -> Result<PasswordHash, PasswordHasherError> {
        Err(PasswordHasherError::HashFailed)
    }
    #[allow(unused_variables)]
    fn verify(&self, password: &str, hash: &PasswordHash) -> Result<bool, PasswordHasherError> {
        Err(PasswordHasherError::VerifyFailed)
    }
}

pub struct FailingUserRepository;

#[async_trait]
#[allow(unused)]
impl UserRepository for FailingUserRepository {
    async fn insert(&self, user: &User) -> Result<(), UserRepositoryError> {
        Err(UserRepositoryError::Persistence(PersistenceError::Failed))
    }

    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, UserRepositoryError> {
        Err(UserRepositoryError::Persistence(PersistenceError::Failed))
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, UserRepositoryError> {
        Err(UserRepositoryError::Persistence(PersistenceError::Failed))
    }

    async fn find_by_ids(&self, ids: &[UserId]) -> Result<Vec<User>, UserRepositoryError> {
        Err(UserRepositoryError::Persistence(PersistenceError::Failed))
    }
}

pub struct MissingUserRepository;

#[async_trait]
#[allow(unused)]
impl UserRepository for MissingUserRepository {
    async fn insert(&self, user: &User) -> Result<(), UserRepositoryError> {
        Err(UserRepositoryError::Persistence(PersistenceError::Failed))
    }

    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, UserRepositoryError> {
        Err(UserRepositoryError::Persistence(PersistenceError::Failed))
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, UserRepositoryError> {
        Err(UserRepositoryError::Persistence(PersistenceError::Failed))
    }

    async fn find_by_ids(&self, ids: &[UserId]) -> Result<Vec<User>, UserRepositoryError> {
        Ok(Vec::new())
    }
}

pub struct FailingHouseholdRepository;

#[async_trait]
#[allow(unused)]
impl HouseholdRepository for FailingHouseholdRepository {
    async fn create_with_owner(
        &self,
        household: &Household,
        owner: &HouseholdMember,
    ) -> Result<(), HouseholdRepositoryError> {
        Err(HouseholdRepositoryError::Persistence(
            PersistenceError::Failed,
        ))
    }

    async fn find_by_id(
        &self,
        id: &HouseholdId,
    ) -> Result<Option<Household>, HouseholdRepositoryError> {
        Err(HouseholdRepositoryError::Persistence(
            PersistenceError::Failed,
        ))
    }

    async fn find_personal_by_owner(
        &self,
        owner: &UserId,
    ) -> Result<Option<Household>, HouseholdRepositoryError> {
        Err(HouseholdRepositoryError::Persistence(
            PersistenceError::Failed,
        ))
    }

    async fn find_for_user(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<Household>, HouseholdRepositoryError> {
        Err(HouseholdRepositoryError::Persistence(
            PersistenceError::Failed,
        ))
    }

    async fn find_member(
        &self,
        household_id: &HouseholdId,
        user_id: &UserId,
    ) -> Result<Option<HouseholdMember>, HouseholdRepositoryError> {
        Err(HouseholdRepositoryError::Persistence(
            PersistenceError::Failed,
        ))
    }

    async fn add_member(&self, member: &HouseholdMember) -> Result<(), HouseholdRepositoryError> {
        Err(HouseholdRepositoryError::Persistence(
            PersistenceError::Failed,
        ))
    }

    async fn find_members(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<HouseholdMember>, HouseholdRepositoryError> {
        Err(HouseholdRepositoryError::Persistence(
            PersistenceError::Failed,
        ))
    }

    async fn remove_member(
        &self,
        household_id: &HouseholdId,
        user_id: &UserId,
    ) -> Result<(), HouseholdRepositoryError> {
        Err(HouseholdRepositoryError::Persistence(
            PersistenceError::Failed,
        ))
    }

    async fn update(&self, household: &Household) -> Result<(), HouseholdRepositoryError> {
        Err(HouseholdRepositoryError::Persistence(
            PersistenceError::Failed,
        ))
    }
}

pub struct DuplicateOnAddHouseholdRepository {
    pub inner: Arc<InMemoryHouseholdRepository>,
}

#[async_trait::async_trait]
impl HouseholdRepository for DuplicateOnAddHouseholdRepository {
    async fn create_with_owner(
        &self,
        household: &Household,
        owner: &HouseholdMember,
    ) -> Result<(), HouseholdRepositoryError> {
        self.inner.create_with_owner(household, owner).await
    }

    async fn find_by_id(
        &self,
        id: &HouseholdId,
    ) -> Result<Option<Household>, HouseholdRepositoryError> {
        self.inner.find_by_id(id).await
    }

    async fn find_personal_by_owner(
        &self,
        owner: &UserId,
    ) -> Result<Option<Household>, HouseholdRepositoryError> {
        self.inner.find_personal_by_owner(owner).await
    }

    async fn find_for_user(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<Household>, HouseholdRepositoryError> {
        self.inner.find_for_user(user_id).await
    }

    async fn find_member(
        &self,
        household_id: &HouseholdId,
        user_id: &UserId,
    ) -> Result<Option<HouseholdMember>, HouseholdRepositoryError> {
        self.inner.find_member(household_id, user_id).await
    }

    #[allow(unused_variables)]
    async fn add_member(&self, member: &HouseholdMember) -> Result<(), HouseholdRepositoryError> {
        Err(HouseholdRepositoryError::MemberAlreadyExists)
    }

    async fn find_members(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<HouseholdMember>, HouseholdRepositoryError> {
        self.inner.find_members(household_id).await
    }

    async fn remove_member(
        &self,
        household_id: &HouseholdId,
        user_id: &UserId,
    ) -> Result<(), HouseholdRepositoryError> {
        self.inner.remove_member(household_id, user_id).await
    }

    async fn update(&self, household: &Household) -> Result<(), HouseholdRepositoryError> {
        self.inner.update(household).await
    }
}

pub struct SharedHouseholdFixture {
    pub owner: User,
    pub household: Household,
}

pub struct MissingOnRemoveHouseholdRepository {
    pub inner: Arc<InMemoryHouseholdRepository>,
}

#[async_trait::async_trait]
impl HouseholdRepository for MissingOnRemoveHouseholdRepository {
    async fn create_with_owner(
        &self,
        household: &Household,
        owner: &HouseholdMember,
    ) -> Result<(), HouseholdRepositoryError> {
        self.inner.create_with_owner(household, owner).await
    }

    async fn find_by_id(
        &self,
        id: &HouseholdId,
    ) -> Result<Option<Household>, HouseholdRepositoryError> {
        self.inner.find_by_id(id).await
    }

    async fn find_personal_by_owner(
        &self,
        owner: &UserId,
    ) -> Result<Option<Household>, HouseholdRepositoryError> {
        self.inner.find_personal_by_owner(owner).await
    }

    async fn find_for_user(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<Household>, HouseholdRepositoryError> {
        self.inner.find_for_user(user_id).await
    }

    async fn find_member(
        &self,
        household_id: &HouseholdId,
        user_id: &UserId,
    ) -> Result<Option<HouseholdMember>, HouseholdRepositoryError> {
        self.inner.find_member(household_id, user_id).await
    }

    async fn add_member(&self, member: &HouseholdMember) -> Result<(), HouseholdRepositoryError> {
        self.inner.add_member(member).await
    }

    async fn find_members(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<HouseholdMember>, HouseholdRepositoryError> {
        self.inner.find_members(household_id).await
    }

    #[allow(unused_variables)]
    async fn remove_member(
        &self,
        household_id: &HouseholdId,
        user_id: &UserId,
    ) -> Result<(), HouseholdRepositoryError> {
        Err(HouseholdRepositoryError::MemberNotFound)
    }

    async fn update(&self, household: &Household) -> Result<(), HouseholdRepositoryError> {
        self.inner.update(household).await
    }
}

pub struct MissingOnUpdateHouseholdRepository {
    pub inner: Arc<InMemoryHouseholdRepository>,
}

#[async_trait::async_trait]
impl HouseholdRepository for MissingOnUpdateHouseholdRepository {
    async fn create_with_owner(
        &self,
        household: &Household,
        owner: &HouseholdMember,
    ) -> Result<(), HouseholdRepositoryError> {
        self.inner.create_with_owner(household, owner).await
    }

    async fn find_by_id(
        &self,
        id: &HouseholdId,
    ) -> Result<Option<Household>, HouseholdRepositoryError> {
        self.inner.find_by_id(id).await
    }

    async fn find_personal_by_owner(
        &self,
        owner: &UserId,
    ) -> Result<Option<Household>, HouseholdRepositoryError> {
        self.inner.find_personal_by_owner(owner).await
    }

    async fn find_for_user(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<Household>, HouseholdRepositoryError> {
        self.inner.find_for_user(user_id).await
    }

    async fn find_member(
        &self,
        household_id: &HouseholdId,
        user_id: &UserId,
    ) -> Result<Option<HouseholdMember>, HouseholdRepositoryError> {
        self.inner.find_member(household_id, user_id).await
    }

    async fn add_member(&self, member: &HouseholdMember) -> Result<(), HouseholdRepositoryError> {
        self.inner.add_member(member).await
    }

    async fn find_members(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<HouseholdMember>, HouseholdRepositoryError> {
        self.inner.find_members(household_id).await
    }

    #[allow(unused_variables)]
    async fn remove_member(
        &self,
        household_id: &HouseholdId,
        user_id: &UserId,
    ) -> Result<(), HouseholdRepositoryError> {
        self.inner.remove_member(household_id, user_id).await
    }

    #[allow(unused_variables)]
    async fn update(&self, household: &Household) -> Result<(), HouseholdRepositoryError> {
        Err(HouseholdRepositoryError::HouseholdNotFound)
    }
}
