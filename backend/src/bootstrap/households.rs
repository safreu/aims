use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    modules::{
        accounts::adapters::PostgresUserRepository,
        households::{
            adapters::{DefaultHouseholdAccessPolicy, PostgresHouseholdRepository},
            application::{
                AddHouseholdMemberService, CreateHouseholdService, GetHouseholdService,
                ListHouseholdMembersService, ListHouseholdsForUserService,
                RemoveHouseholdMemberService, RenameHouseholdService,
                SubscribeHouseholdEventsService,
            },
            ports::{HouseholdEventPublisher, HouseholdEventSubscriber},
        },
    },
    shared::api::HouseholdsState,
};

pub(super) fn build_households_state(
    pool: &PgPool,
    household_events_publisher: Arc<dyn HouseholdEventPublisher>,
    household_events_subscriber: Arc<dyn HouseholdEventSubscriber>,
) -> HouseholdsState {
    let household_repository: Arc<PostgresHouseholdRepository> =
        Arc::new(PostgresHouseholdRepository::new(pool.clone()));
    let user_repository = Arc::new(PostgresUserRepository::new(pool.clone()));

    let create_household_service =
        Arc::new(CreateHouseholdService::new(household_repository.clone()));

    let list_households_for_user_service = Arc::new(ListHouseholdsForUserService::new(
        household_repository.clone(),
    ));

    let household_access_policy = Arc::new(DefaultHouseholdAccessPolicy::new(
        household_repository.clone(),
    ));

    let get_household_for_user_service = Arc::new(GetHouseholdService::new(
        household_repository.clone(),
        household_access_policy.clone(),
    ));

    let add_household_member_service = Arc::new(AddHouseholdMemberService::new(
        household_repository.clone(),
        user_repository.clone(),
        household_events_publisher.clone(),
    ));

    let list_household_members_service = Arc::new(ListHouseholdMembersService::new(
        household_repository.clone(),
        household_access_policy.clone(),
        user_repository,
    ));

    let remove_household_member_service = Arc::new(RemoveHouseholdMemberService::new(
        household_repository.clone(),
        household_access_policy.clone(),
        household_events_publisher.clone(),
    ));

    let rename_household_service = Arc::new(RenameHouseholdService::new(
        household_repository,
        household_events_publisher.clone(),
    ));

    let subscribe_household_event_service = Arc::new(SubscribeHouseholdEventsService::new(
        household_access_policy.clone(),
        household_events_subscriber.clone(),
    ));

    HouseholdsState {
        create_household: create_household_service,
        list_households_for_user: list_households_for_user_service,
        get_household_for_user: get_household_for_user_service,
        add_household_member: add_household_member_service,
        list_household_members: list_household_members_service,
        remove_household_member: remove_household_member_service,
        rename_household: rename_household_service,

        subscribe_household_events: subscribe_household_event_service,
    }
}
