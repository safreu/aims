use std::sync::Arc;

use crate::modules::households::application::{
    AddHouseholdMemberService, CreateHouseholdService, GetHouseholdService,
    ListHouseholdMembersService, ListHouseholdsForUserService, RemoveHouseholdMemberService,
    RenameHouseholdService, SubscribeHouseholdEventsService,
};

#[derive(Clone)]
pub struct HouseholdsState {
    pub create_household: Arc<CreateHouseholdService>,
    pub list_households_for_user: Arc<ListHouseholdsForUserService>,
    pub get_household_for_user: Arc<GetHouseholdService>,
    pub add_household_member: Arc<AddHouseholdMemberService>,
    pub list_household_members: Arc<ListHouseholdMembersService>,
    pub remove_household_member: Arc<RemoveHouseholdMemberService>,
    pub rename_household: Arc<RenameHouseholdService>,

    pub subscribe_household_events: Arc<SubscribeHouseholdEventsService>,
}
