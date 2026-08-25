use async_trait::async_trait;

use crate::{
    modules::{
        households::domain::HouseholdId,
        shopping::domain::{CustomShoppingEntry, CustomShoppingEntryId},
    },
    shared::db::PersistenceError,
};

#[async_trait]
pub trait CustomShoppingEntryRepository: Send + Sync {
    async fn insert(
        &self,
        entry: &CustomShoppingEntry,
    ) -> Result<(), CustomShoppingEntryRepositoryError>;

    async fn find_by_id_for_household(
        &self,
        entry_id: &CustomShoppingEntryId,
        household_id: &HouseholdId,
    ) -> Result<Option<CustomShoppingEntry>, CustomShoppingEntryRepositoryError>;

    async fn find_for_household(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<CustomShoppingEntry>, CustomShoppingEntryRepositoryError>;

    async fn update(
        &self,
        entry: &CustomShoppingEntry,
    ) -> Result<(), CustomShoppingEntryRepositoryError>;

    async fn delete(
        &self,
        entry_id: &CustomShoppingEntryId,
        household_id: &HouseholdId,
    ) -> Result<(), CustomShoppingEntryRepositoryError>;
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CustomShoppingEntryRepositoryError {
    #[error("Custom shopping entry not found")]
    EntryNotFound,
    #[error("Invalid custom shopping entry data stored")]
    InvalidStoredData,
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}
