use crate::modules::households::domain::HouseholdId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HouseholdApplicationEvent {
    Changed { household_id: HouseholdId },
}
