use crate::modules::{accounts::domain::UserId, households::domain::HouseholdId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HouseholdApplicationEvent {
    Created {
        household_id: HouseholdId,
        owner_id: UserId,
    },
    NameUpdated {
        household_id: HouseholdId,
    },
    Deleted {
        household_id: HouseholdId,
        owner_id: UserId,
    },
    MemberAdded {
        household_id: HouseholdId,
        member_id: UserId,
    },
    MemberRemoved {
        household_id: HouseholdId,
        member_id: UserId,
    },
    MemberLeft {
        household_id: HouseholdId,
        member_id: UserId,
    },
}
