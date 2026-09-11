use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::modules::{
    accounts::domain::UserId,
    households::{
        adapters::validate::validate_aggregate,
        domain::{Household, HouseholdId, HouseholdKind, HouseholdMember},
        ports::{HouseholdRepository, HouseholdRepositoryError},
    },
};

struct HouseholdState {
    households: HashMap<HouseholdId, Household>,
    members: HashMap<(HouseholdId, UserId), HouseholdMember>,
}

pub struct InMemoryHouseholdRepository {
    state: RwLock<HouseholdState>,
}

impl InMemoryHouseholdRepository {
    pub fn new() -> Self {
        let households: HashMap<HouseholdId, Household> = HashMap::new();
        let members: HashMap<(HouseholdId, UserId), HouseholdMember> = HashMap::new();

        let state = HouseholdState {
            households,
            members,
        };
        Self {
            state: RwLock::new(state),
        }
    }
}

impl Default for InMemoryHouseholdRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HouseholdRepository for InMemoryHouseholdRepository {
    async fn create_with_owner(
        &self,
        household: &Household,
        owner: &HouseholdMember,
    ) -> Result<(), HouseholdRepositoryError> {
        let mut state = self.state.write().await;

        validate_aggregate(household, owner)?;

        if household.kind() == HouseholdKind::Personal {
            let already_exists = state.households.values().any(|existing| {
                existing.kind() == HouseholdKind::Personal
                    && existing.personal_owner_id() == Some(owner.user_id())
            });
            if already_exists {
                return Err(HouseholdRepositoryError::PersonalHouseholdAlreadyExists);
            }
        }

        if state.households.contains_key(&household.id()) {
            return Err(HouseholdRepositoryError::HouseholdAlreadyExists);
        }

        state.households.insert(household.id(), household.clone());

        state
            .members
            .insert((owner.household_id(), owner.user_id()), owner.clone());

        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &HouseholdId,
    ) -> Result<Option<Household>, HouseholdRepositoryError> {
        let state = self.state.read().await;
        Ok(state.households.get(id).cloned())
    }

    async fn find_personal_by_owner(
        &self,
        owner: &UserId,
    ) -> Result<Option<Household>, HouseholdRepositoryError> {
        let state = self.state.read().await;

        let household = state
            .households
            .values()
            .find(|household| {
                household.kind() == HouseholdKind::Personal
                    && household.personal_owner_id() == Some(*owner)
            })
            .cloned();

        Ok(household)
    }

    async fn find_for_user(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<Household>, HouseholdRepositoryError> {
        let state = self.state.read().await;

        let households = state
            .members
            .values()
            .filter(|member| member.user_id() == *user_id)
            .filter_map(|member| state.households.get(&member.household_id()).cloned())
            .collect();

        Ok(households)
    }

    async fn find_member(
        &self,
        household_id: &HouseholdId,
        user_id: &UserId,
    ) -> Result<Option<HouseholdMember>, HouseholdRepositoryError> {
        let state = self.state.read().await;

        let member = state.members.get(&(*household_id, *user_id));

        Ok(member.cloned())
    }

    async fn add_member(&self, member: &HouseholdMember) -> Result<(), HouseholdRepositoryError> {
        let mut state = self.state.write().await;

        if !state.households.contains_key(&member.household_id()) {
            return Err(HouseholdRepositoryError::HouseholdNotFound);
        }

        let key = (member.household_id(), member.user_id());

        if state.members.contains_key(&key) {
            return Err(HouseholdRepositoryError::MemberAlreadyExists);
        }

        state.members.insert(key, member.clone());
        Ok(())
    }

    async fn find_members(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<HouseholdMember>, HouseholdRepositoryError> {
        let state = self.state.read().await;

        let members = state
            .members
            .values()
            .filter(|member| member.household_id() == *household_id)
            .cloned()
            .collect();

        Ok(members)
    }

    async fn remove_member(
        &self,
        household_id: &HouseholdId,
        user_id: &UserId,
    ) -> Result<(), HouseholdRepositoryError> {
        let mut state = self.state.write().await;

        let key = (*household_id, *user_id);

        if state.members.remove(&key).is_none() {
            return Err(HouseholdRepositoryError::MemberNotFound);
        }

        Ok(())
    }

    async fn update(&self, household: &Household) -> Result<(), HouseholdRepositoryError> {
        let mut state = self.state.write().await;

        if !state.households.contains_key(&household.id()) {
            return Err(HouseholdRepositoryError::HouseholdNotFound);
        }

        state.households.insert(household.id(), household.clone());

        Ok(())
    }

    async fn delete(&self, _household_id: &HouseholdId) -> Result<(), HouseholdRepositoryError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use chrono::Utc;

    use crate::modules::households::domain::{HouseholdError, HouseholdName, HouseholdRole};

    use super::*;

    fn create_household(
        id: HouseholdId,
        kind: HouseholdKind,
        personal_owner_id: Option<UserId>,
    ) -> Result<Household, HouseholdError> {
        Household::new(
            id,
            HouseholdName::parse("This is a name").expect("Test name should be valid"),
            kind,
            personal_owner_id,
            Utc::now(),
            Utc::now(),
        )
    }

    fn create_owned_household(
        user_id: UserId,
        kind: HouseholdKind,
    ) -> (Household, HouseholdMember) {
        let household_id = HouseholdId::new();
        let now = Utc::now();

        let owner = HouseholdMember::new(household_id, user_id, HouseholdRole::Owner, now);

        let personal_owner_id = match kind {
            HouseholdKind::Personal => Some(user_id),
            HouseholdKind::Shared => None,
        };

        let household = Household::new(
            household_id,
            HouseholdName::parse("Test household").expect("Test household name should be valid"),
            kind,
            personal_owner_id,
            now,
            now,
        )
        .expect("Test household should be valid");

        (household, owner)
    }

    fn create_household_member(
        household_id: HouseholdId,
        user_id: UserId,
        role: HouseholdRole,
    ) -> HouseholdMember {
        HouseholdMember::new(household_id, user_id, role, Utc::now())
    }

    #[tokio::test]
    async fn personal_household_with_owner_can_be_created() {
        let repository = InMemoryHouseholdRepository::new();

        let (household, owner) = create_owned_household(UserId::new(), HouseholdKind::Personal);

        let result = repository.create_with_owner(&household, &owner).await;

        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn shared_household_with_owner_can_be_created() {
        let repository = InMemoryHouseholdRepository::new();

        let (household, owner) = create_owned_household(UserId::new(), HouseholdKind::Shared);

        let result = repository.create_with_owner(&household, &owner).await;

        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn created_household_can_be_found_by_id() {
        let repository = InMemoryHouseholdRepository::new();

        let (household, owner) = create_owned_household(UserId::new(), HouseholdKind::Shared);

        repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let result = repository
            .find_by_id(&household.id())
            .await
            .expect("Household lookup should succeed");

        assert_eq!(result, Some(household))
    }

    #[tokio::test]
    async fn personal_household_can_be_found_by_owner() {
        let repository = InMemoryHouseholdRepository::new();

        let (household, owner) = create_owned_household(UserId::new(), HouseholdKind::Personal);

        repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let result = repository
            .find_personal_by_owner(&owner.user_id())
            .await
            .expect("Household lookup should succeed");

        assert_eq!(result, Some(household))
    }

    #[tokio::test]
    async fn households_for_user_are_returned() {
        let repository = InMemoryHouseholdRepository::new();

        let user_id = UserId::new();

        let (personal_household, personal_owner) =
            create_owned_household(user_id, HouseholdKind::Personal);
        let (shared_household, shared_owner) =
            create_owned_household(user_id, HouseholdKind::Shared);

        repository
            .create_with_owner(&personal_household, &personal_owner)
            .await
            .expect("Personal household creation should succeed");
        repository
            .create_with_owner(&shared_household, &shared_owner)
            .await
            .expect("Shared household creation should succeed");

        let result = repository
            .find_for_user(&personal_owner.user_id())
            .await
            .expect("Household lookup should succeed");

        assert_eq!(result.len(), 2);
        assert!(result.contains(&personal_household));
        assert!(result.contains(&shared_household))
    }

    #[tokio::test]
    async fn duplicate_personal_household_is_rejected() {
        let repository = InMemoryHouseholdRepository::new();

        let user_id = UserId::new();

        let (personal_household, first_owner) =
            create_owned_household(user_id, HouseholdKind::Personal);
        let (another_personal_household, second_owner) =
            create_owned_household(user_id, HouseholdKind::Personal);

        repository
            .create_with_owner(&personal_household, &first_owner)
            .await
            .expect("Household creation should succeed");

        let result = repository
            .create_with_owner(&another_personal_household, &second_owner)
            .await;

        assert_eq!(
            result,
            Err(HouseholdRepositoryError::PersonalHouseholdAlreadyExists)
        )
    }

    #[tokio::test]
    async fn inconsistent_owner_membership_is_rejected() {
        let repository = InMemoryHouseholdRepository::new();

        let user_id = UserId::new();
        let household_id = HouseholdId::new();
        let another_household_id = HouseholdId::new();

        let household = create_household(household_id, HouseholdKind::Shared, None)
            .expect("Test household should be valid");

        let owner = create_household_member(another_household_id, user_id, HouseholdRole::Owner);

        let result = repository.create_with_owner(&household, &owner).await;

        assert_eq!(result, Err(HouseholdRepositoryError::InvalidAggregate))
    }

    #[tokio::test]
    async fn non_owner_membership_is_rejected() {
        let repository = InMemoryHouseholdRepository::new();

        let user_id = UserId::new();
        let household_id = HouseholdId::new();

        let household = create_household(household_id, HouseholdKind::Shared, None)
            .expect("Test household should be valid");

        let owner = create_household_member(household_id, user_id, HouseholdRole::Member);

        let result = repository.create_with_owner(&household, &owner).await;

        assert_eq!(result, Err(HouseholdRepositoryError::InvalidAggregate))
    }

    #[tokio::test]
    async fn unknown_household_returns_none() {
        let repository = InMemoryHouseholdRepository::default();

        let result = repository
            .find_by_id(&HouseholdId::new())
            .await
            .expect("Household lookup should succeed");

        assert!(result.is_none())
    }

    #[tokio::test]
    async fn user_without_households_returns_empty_list() {
        let repository = InMemoryHouseholdRepository::default();

        let result = repository
            .find_for_user(&UserId::new())
            .await
            .expect("Household lookup should succeed");

        assert!(result.is_empty())
    }

    #[tokio::test]
    async fn existing_member_ship_is_returned() {
        let repository = InMemoryHouseholdRepository::default();

        let (household, owner) = create_owned_household(UserId::new(), HouseholdKind::Shared);

        repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let result = repository
            .find_member(&household.id(), &owner.user_id())
            .await
            .expect("Household lookup should succeed");

        assert_eq!(result, Some(owner))
    }

    #[tokio::test]
    async fn unknown_member_returns_none() {
        let repository = InMemoryHouseholdRepository::default();

        let household_id = HouseholdId::new();
        let user_id = UserId::new();

        let result = repository
            .find_member(&household_id, &user_id)
            .await
            .expect("Household lookup should succeed");

        assert!(result.is_none())
    }

    #[tokio::test]
    async fn member_lookup_is_scoped_to_household() {
        let repository = InMemoryHouseholdRepository::default();

        let user_id = UserId::new();

        let (household, owner) = create_owned_household(user_id, HouseholdKind::Shared);

        repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let result = repository
            .find_member(&HouseholdId::new(), &owner.user_id())
            .await
            .expect("Household lookup should succeed");

        assert!(result.is_none())
    }

    #[tokio::test]
    async fn member_can_be_added_to_existing_household() {
        let repository = InMemoryHouseholdRepository::default();

        let owner_id = UserId::new();
        let member_id = UserId::new();

        let (household, owner) = create_owned_household(owner_id, HouseholdKind::Shared);

        repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let member =
            HouseholdMember::new(household.id(), member_id, HouseholdRole::Member, Utc::now());

        let result = repository.add_member(&member).await;

        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn added_member_can_be_found() {
        let repository = InMemoryHouseholdRepository::default();

        let owner_id = UserId::new();
        let member_id = UserId::new();

        let (household, owner) = create_owned_household(owner_id, HouseholdKind::Shared);

        repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let member =
            HouseholdMember::new(household.id(), member_id, HouseholdRole::Member, Utc::now());

        repository
            .add_member(&member)
            .await
            .expect("Adding member should succeed");

        let stored = repository
            .find_member(&household.id(), &member_id)
            .await
            .expect("Membership lookup should succeed");

        assert_eq!(stored, Some(member))
    }

    #[tokio::test]
    async fn duplicate_member_is_rejected() {
        let repository = InMemoryHouseholdRepository::default();

        let owner_id = UserId::new();
        let member_id = UserId::new();

        let (household, owner) = create_owned_household(owner_id, HouseholdKind::Shared);

        repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let member =
            HouseholdMember::new(household.id(), member_id, HouseholdRole::Member, Utc::now());

        repository
            .add_member(&member)
            .await
            .expect("Adding member should succeed");

        let result = repository.add_member(&member).await;

        assert_eq!(result, Err(HouseholdRepositoryError::MemberAlreadyExists))
    }

    #[tokio::test]
    async fn member_cannot_be_added_to_unknown_household() {
        let repository = InMemoryHouseholdRepository::default();

        let member = HouseholdMember::new(
            HouseholdId::new(),
            UserId::new(),
            HouseholdRole::Member,
            Utc::now(),
        );

        let result = repository.add_member(&member).await;

        assert_eq!(result, Err(HouseholdRepositoryError::HouseholdNotFound))
    }

    #[tokio::test]
    async fn find_members_returns_all_members_of_household() {
        let repository = InMemoryHouseholdRepository::default();

        let (household, owner) = create_owned_household(UserId::new(), HouseholdKind::Shared);

        repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let member_id = UserId::new();

        let member =
            HouseholdMember::new(household.id(), member_id, HouseholdRole::Member, Utc::now());

        repository
            .add_member(&member)
            .await
            .expect("Adding member should succeed");

        let result = repository
            .find_members(&household.id())
            .await
            .expect("Members lookup should succeed");

        assert_eq!(result.len(), 2);
        assert!(result.contains(&owner));
        assert!(result.contains(&member))
    }

    #[tokio::test]
    async fn find_members_does_not_return_members_of_other_households() {
        let repository = InMemoryHouseholdRepository::default();

        let (household, owner) = create_owned_household(UserId::new(), HouseholdKind::Shared);

        let (another_household, another_owner) =
            create_owned_household(UserId::new(), HouseholdKind::Shared);

        repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        repository
            .create_with_owner(&another_household, &another_owner)
            .await
            .expect("Household creation should succeed");

        let member_id = UserId::new();

        let member =
            HouseholdMember::new(household.id(), member_id, HouseholdRole::Member, Utc::now());

        repository
            .add_member(&member)
            .await
            .expect("Adding member should succeed");

        let result = repository
            .find_members(&household.id())
            .await
            .expect("Members lookup should succeed");

        assert_eq!(result.len(), 2);
        assert!(
            result
                .iter()
                .all(|member| member.household_id() == household.id())
        )
    }

    #[tokio::test]
    async fn find_members_for_unknown_household_returns_empty_vec() {
        let repository = InMemoryHouseholdRepository::default();

        let result = repository
            .find_members(&HouseholdId::new())
            .await
            .expect("Members lookup should succeed");

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn existing_member_can_be_removed() {
        let repository = InMemoryHouseholdRepository::default();

        let owner_id = UserId::new();
        let member_id = UserId::new();

        let (household, owner) = create_owned_household(owner_id, HouseholdKind::Shared);

        repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let member =
            HouseholdMember::new(household.id(), member_id, HouseholdRole::Member, Utc::now());

        repository
            .add_member(&member)
            .await
            .expect("Adding member should succeed");

        let result = repository.remove_member(&household.id(), &member_id).await;

        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn removed_member_can_no_longer_be_found() {
        let repository = InMemoryHouseholdRepository::default();

        let owner_id = UserId::new();
        let member_id = UserId::new();

        let (household, owner) = create_owned_household(owner_id, HouseholdKind::Shared);

        repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let member =
            HouseholdMember::new(household.id(), member_id, HouseholdRole::Member, Utc::now());

        repository
            .add_member(&member)
            .await
            .expect("Adding member should succeed");

        repository
            .remove_member(&household.id(), &member_id)
            .await
            .expect("Removing member should succeed");

        let result = repository
            .find_member(&household.id(), &member_id)
            .await
            .expect("Membership lookup should succeed");

        assert!(result.is_none())
    }

    #[tokio::test]
    async fn removing_unknown_member_returns_not_found() {
        let repository = InMemoryHouseholdRepository::default();

        let owner_id = UserId::new();
        let member_id = UserId::new();

        let (household, owner) = create_owned_household(owner_id, HouseholdKind::Shared);

        repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let result = repository.remove_member(&household.id(), &member_id).await;

        assert_eq!(result, Err(HouseholdRepositoryError::MemberNotFound))
    }

    #[tokio::test]
    async fn updated_household_can_be_loaded() {
        let repository = InMemoryHouseholdRepository::new();

        let (mut household, owner) = create_owned_household(UserId::new(), HouseholdKind::Shared);

        repository
            .create_with_owner(&household, &owner)
            .await
            .expect("Household creation should succeed");

        let new_name =
            HouseholdName::parse("Updated household").expect("Test household name should be valid");

        household.rename(new_name, Utc::now());

        repository
            .update(&household)
            .await
            .expect("Household update should succeed");

        let stored = repository
            .find_by_id(&household.id())
            .await
            .expect("Household lookup should succeed")
            .expect("Household should succeed");

        assert_eq!(stored, household)
    }

    #[tokio::test]
    async fn updating_unknown_household_returns_household_not_found() {
        let repository = InMemoryHouseholdRepository::new();

        let owner_id = UserId::new();

        let (mut household, _) = create_owned_household(owner_id, HouseholdKind::Shared);

        let new_name =
            HouseholdName::parse("Updated household").expect("Test household name should be valid");

        household.rename(new_name, Utc::now());

        let result = repository.update(&household).await;

        assert_eq!(result, Err(HouseholdRepositoryError::HouseholdNotFound))
    }
}
