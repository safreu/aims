use async_trait::async_trait;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::domain::{Household, HouseholdId, HouseholdMember},
    },
    shared::db::PersistenceError,
};

#[async_trait]
pub trait HouseholdRepository: Send + Sync {
    async fn create_with_owner(
        &self,
        household: &Household,
        owner: &HouseholdMember,
    ) -> Result<(), HouseholdRepositoryError>;

    async fn find_by_id(
        &self,
        id: &HouseholdId,
    ) -> Result<Option<Household>, HouseholdRepositoryError>;

    async fn find_personal_by_owner(
        &self,
        owner: &UserId,
    ) -> Result<Option<Household>, HouseholdRepositoryError>;

    async fn find_for_user(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<Household>, HouseholdRepositoryError>;

    async fn find_member(
        &self,
        household_id: &HouseholdId,
        user_id: &UserId,
    ) -> Result<Option<HouseholdMember>, HouseholdRepositoryError>;

    async fn add_member(&self, member: &HouseholdMember) -> Result<(), HouseholdRepositoryError>;

    async fn find_members(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<HouseholdMember>, HouseholdRepositoryError>;

    async fn remove_member(
        &self,
        household_id: &HouseholdId,
        user_id: &UserId,
    ) -> Result<(), HouseholdRepositoryError>;

    async fn update(&self, household: &Household) -> Result<(), HouseholdRepositoryError>;

    async fn delete(&self, household_id: &HouseholdId) -> Result<(), HouseholdRepositoryError>;
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum HouseholdRepositoryError {
    #[error("A personal household already exists for this owner")]
    PersonalHouseholdAlreadyExists,
    #[error("Stored household data is invalid")]
    InvalidStoredData,
    #[error("Household an owner membership are inconsistent")]
    InvalidAggregate,
    #[error("Household already exists")]
    HouseholdAlreadyExists,
    #[error("Household member already exists")]
    MemberAlreadyExists,
    #[error("Household does not exist")]
    HouseholdNotFound,
    #[error("Household member was not found")]
    MemberNotFound,
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}
