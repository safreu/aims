use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::modules::{
    households::domain::HouseholdId,
    scanning::{
        domain::{QrAction, QrActionId},
        ports::{QrActionRepository, QrActionRepositoryError},
    },
};

pub struct InMemoryQrActionRepository {
    actions: RwLock<HashMap<QrActionId, QrAction>>,
}

impl InMemoryQrActionRepository {
    pub fn new() -> Self {
        Self {
            actions: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryQrActionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl QrActionRepository for InMemoryQrActionRepository {
    async fn insert(&self, action: &QrAction) -> Result<(), QrActionRepositoryError> {
        let mut actions = self.actions.write().await;

        actions.insert(action.id(), action.clone());

        Ok(())
    }

    async fn find_by_id_for_household(
        &self,
        action_id: &QrActionId,
        household_id: &HouseholdId,
    ) -> Result<Option<QrAction>, QrActionRepositoryError> {
        let actions = self.actions.read().await;

        Ok(actions
            .get(action_id)
            .filter(|action| action.household_id() == *household_id)
            .cloned())
    }

    async fn find_active_for_household(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<QrAction>, QrActionRepositoryError> {
        let actions = self.actions.read().await;

        Ok(actions
            .values()
            .filter(|action| action.household_id() == *household_id && !action.is_revoked())
            .cloned()
            .collect())
    }

    async fn revoke(&self, action: &QrAction) -> Result<(), QrActionRepositoryError> {
        let mut actions = self.actions.write().await;

        let existing = actions
            .get(&action.id())
            .ok_or(QrActionRepositoryError::QrActionNotFound)?;

        if existing.household_id() != action.household_id() {
            return Err(QrActionRepositoryError::QrActionNotFound);
        }

        if existing.is_revoked() {
            return Err(QrActionRepositoryError::QrActionRevoked);
        }

        actions.insert(action.id(), action.clone());
        todo!()
    }
}
