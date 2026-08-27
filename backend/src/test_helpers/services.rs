use std::sync::Arc;

use chrono::Duration;

use crate::{
    modules::{
        accounts::{
            adapters::{Argon2PasswordHasher, InMemorySessionRepository, InMemoryUserRepository},
            application::{
                AuthenticateSessionService, CreateSessionService, LoginUserService,
                RegisterUserService,
            },
        },
        households::{
            adapters::{
                BroadcastHouseholdEvents, DefaultHouseholdAccessPolicy, InMemoryHouseholdRepository,
            },
            application::{
                AddHouseholdMemberService, CreateHouseholdService, GetHouseholdService,
                ListHouseholdMembersService, ListHouseholdsForUserService,
                RemoveHouseholdMemberService, RenameHouseholdService,
            },
            ports::HouseholdEventPublisher,
        },
        inventory::{
            adapters::{InMemoryCategoryRepository, InMemoryInventoryItemRepository},
            application::{
                ArchiveInventoryItemService, CreateCategoryService, CreateInventoryItemService,
                DeleteCategoryService, ListCategoriesService, RestoreInventoryItemService,
                UpdateInventoryItemService,
            },
        },
    },
    shared::auth::Sha256TokenHasher,
    test_helpers::FixedSessionTokenGenerator,
};

pub fn build_auth_service() -> (
    AuthenticateSessionService,
    Arc<InMemorySessionRepository>,
    Arc<Sha256TokenHasher>,
) {
    let repository = Arc::new(InMemorySessionRepository::new());
    let hasher = Arc::new(Sha256TokenHasher);

    let service = AuthenticateSessionService::new(repository.clone(), hasher.clone());

    (service, repository, hasher)
}

pub fn build_register_service() -> (
    RegisterUserService,
    Arc<InMemoryUserRepository>,
    Arc<Argon2PasswordHasher>,
) {
    let repository = Arc::new(InMemoryUserRepository::new());
    let hasher = Arc::new(Argon2PasswordHasher::new());

    let service = RegisterUserService::new(repository.clone(), hasher.clone());

    (service, repository, hasher)
}

pub fn build_create_session_service() -> (
    CreateSessionService,
    Arc<InMemorySessionRepository>,
    Arc<Sha256TokenHasher>,
) {
    let repository = Arc::new(InMemorySessionRepository::new());

    let generator = Arc::new(FixedSessionTokenGenerator::new(
        "this-session-token-is-fixed",
    ));

    let hasher = Arc::new(Sha256TokenHasher::new());

    let service = CreateSessionService::new(
        repository.clone(),
        generator,
        hasher.clone(),
        Duration::hours(1),
    );

    (service, repository, hasher)
}

pub fn build_login_service() -> (
    LoginUserService,
    Arc<InMemoryUserRepository>,
    Arc<Argon2PasswordHasher>,
) {
    let repository = Arc::new(InMemoryUserRepository::new());
    let hasher = Arc::new(Argon2PasswordHasher::new());

    let service = LoginUserService::new(repository.clone(), hasher.clone());

    (service, repository, hasher)
}

pub fn build_create_household_service() -> (CreateHouseholdService, Arc<InMemoryHouseholdRepository>)
{
    let repository = Arc::new(InMemoryHouseholdRepository::new());

    let service = CreateHouseholdService::new(repository.clone());

    (service, repository)
}

pub fn build_list_households_service() -> (
    ListHouseholdsForUserService,
    Arc<InMemoryHouseholdRepository>,
) {
    let repository = Arc::new(InMemoryHouseholdRepository::new());

    let service = ListHouseholdsForUserService::new(repository.clone());

    (service, repository)
}

pub fn build_get_household_service() -> (GetHouseholdService, Arc<InMemoryHouseholdRepository>) {
    let repository = Arc::new(InMemoryHouseholdRepository::new());
    let policy = Arc::new(DefaultHouseholdAccessPolicy::new(repository.clone()));

    let service = GetHouseholdService::new(repository.clone(), policy);

    (service, repository)
}

pub fn build_add_member_service() -> (
    AddHouseholdMemberService,
    Arc<InMemoryHouseholdRepository>,
    Arc<InMemoryUserRepository>,
) {
    let household_repository = Arc::new(InMemoryHouseholdRepository::new());
    let user_repository = Arc::new(InMemoryUserRepository::new());

    let service =
        AddHouseholdMemberService::new(household_repository.clone(), user_repository.clone());

    (service, household_repository, user_repository)
}

pub fn build_list_household_members_service() -> (
    ListHouseholdMembersService,
    Arc<InMemoryHouseholdRepository>,
    Arc<InMemoryUserRepository>,
) {
    let household_repository = Arc::new(InMemoryHouseholdRepository::new());
    let policy = Arc::new(DefaultHouseholdAccessPolicy::new(
        household_repository.clone(),
    ));
    let user_repository = Arc::new(InMemoryUserRepository::new());

    let service = ListHouseholdMembersService::new(
        household_repository.clone(),
        policy,
        user_repository.clone(),
    );

    (service, household_repository, user_repository)
}

pub fn build_remove_household_member_service() -> (
    RemoveHouseholdMemberService,
    Arc<InMemoryHouseholdRepository>,
    Arc<InMemoryUserRepository>,
) {
    let household_repository = Arc::new(InMemoryHouseholdRepository::new());
    let policy = Arc::new(DefaultHouseholdAccessPolicy::new(
        household_repository.clone(),
    ));
    let user_repository = Arc::new(InMemoryUserRepository::new());

    let service = RemoveHouseholdMemberService::new(household_repository.clone(), policy);

    (service, household_repository, user_repository)
}

pub fn build_rename_household_service() -> (RenameHouseholdService, Arc<InMemoryHouseholdRepository>)
{
    let repository = Arc::new(InMemoryHouseholdRepository::new());

    let service = RenameHouseholdService::new(repository.clone());

    (service, repository)
}

pub fn build_create_inventory_item_service() -> (
    CreateInventoryItemService,
    Arc<InMemoryInventoryItemRepository>,
    Arc<InMemoryCategoryRepository>,
    Arc<InMemoryHouseholdRepository>,
) {
    let household_repository = Arc::new(InMemoryHouseholdRepository::new());
    let household_access_policy = Arc::new(DefaultHouseholdAccessPolicy::new(
        household_repository.clone(),
    ));
    let category_repository = Arc::new(InMemoryCategoryRepository::new());
    let inventory_item_repository = Arc::new(InMemoryInventoryItemRepository::new());

    let household_events = Arc::new(BroadcastHouseholdEvents::new(64));
    let household_events_publisher: Arc<dyn HouseholdEventPublisher> = household_events.clone();

    let service = CreateInventoryItemService::new(
        household_access_policy,
        category_repository.clone(),
        inventory_item_repository.clone(),
        household_events_publisher.clone(),
    );

    (
        service,
        inventory_item_repository,
        category_repository,
        household_repository,
    )
}

pub fn build_create_category_service() -> (
    CreateCategoryService,
    Arc<InMemoryCategoryRepository>,
    Arc<InMemoryHouseholdRepository>,
) {
    let household_repository = Arc::new(InMemoryHouseholdRepository::new());
    let household_access_policy = Arc::new(DefaultHouseholdAccessPolicy::new(
        household_repository.clone(),
    ));
    let category_repository = Arc::new(InMemoryCategoryRepository::new());

    let service = CreateCategoryService::new(household_access_policy, category_repository.clone());

    (service, category_repository, household_repository)
}

pub fn build_list_categories_service() -> (
    ListCategoriesService,
    Arc<InMemoryCategoryRepository>,
    Arc<InMemoryHouseholdRepository>,
) {
    let household_repository = Arc::new(InMemoryHouseholdRepository::new());
    let household_access_policy = Arc::new(DefaultHouseholdAccessPolicy::new(
        household_repository.clone(),
    ));
    let category_repository = Arc::new(InMemoryCategoryRepository::new());

    let service = ListCategoriesService::new(household_access_policy, category_repository.clone());

    (service, category_repository, household_repository)
}

pub fn build_delete_category_service() -> (
    DeleteCategoryService,
    Arc<InMemoryCategoryRepository>,
    Arc<InMemoryHouseholdRepository>,
) {
    let household_repository = Arc::new(InMemoryHouseholdRepository::new());
    let household_access_policy = Arc::new(DefaultHouseholdAccessPolicy::new(
        household_repository.clone(),
    ));
    let category_repository = Arc::new(InMemoryCategoryRepository::new());

    let service = DeleteCategoryService::new(household_access_policy, category_repository.clone());

    (service, category_repository, household_repository)
}

pub fn build_update_inventory_item_service() -> (
    UpdateInventoryItemService,
    Arc<InMemoryCategoryRepository>,
    Arc<InMemoryInventoryItemRepository>,
    Arc<InMemoryHouseholdRepository>,
) {
    let household_repository = Arc::new(InMemoryHouseholdRepository::new());
    let household_access_policy = Arc::new(DefaultHouseholdAccessPolicy::new(
        household_repository.clone(),
    ));
    let category_repository = Arc::new(InMemoryCategoryRepository::new());
    let inventory_item_repository = Arc::new(InMemoryInventoryItemRepository::new());

    let household_events = Arc::new(BroadcastHouseholdEvents::new(64));
    let household_events_publisher: Arc<dyn HouseholdEventPublisher> = household_events.clone();

    let service = UpdateInventoryItemService::new(
        household_access_policy,
        category_repository.clone(),
        inventory_item_repository.clone(),
        household_events_publisher.clone(),
    );

    (
        service,
        category_repository,
        inventory_item_repository,
        household_repository,
    )
}

pub fn build_archive_item_service() -> (
    ArchiveInventoryItemService,
    Arc<InMemoryInventoryItemRepository>,
    Arc<InMemoryHouseholdRepository>,
) {
    let household_repository = Arc::new(InMemoryHouseholdRepository::new());
    let household_access_policy = Arc::new(DefaultHouseholdAccessPolicy::new(
        household_repository.clone(),
    ));
    let inventory_item_repository = Arc::new(InMemoryInventoryItemRepository::new());

    let household_events = Arc::new(BroadcastHouseholdEvents::new(64));
    let household_events_publisher: Arc<dyn HouseholdEventPublisher> = household_events.clone();

    let service = ArchiveInventoryItemService::new(
        household_access_policy,
        inventory_item_repository.clone(),
        household_events_publisher.clone(),
    );

    (service, inventory_item_repository, household_repository)
}

pub fn build_restore_item_service() -> (
    RestoreInventoryItemService,
    Arc<InMemoryInventoryItemRepository>,
    Arc<InMemoryHouseholdRepository>,
) {
    let household_repository = Arc::new(InMemoryHouseholdRepository::new());
    let household_access_policy = Arc::new(DefaultHouseholdAccessPolicy::new(
        household_repository.clone(),
    ));
    let inventory_item_repository = Arc::new(InMemoryInventoryItemRepository::new());

    let household_events = Arc::new(BroadcastHouseholdEvents::new(64));
    let household_events_publisher: Arc<dyn HouseholdEventPublisher> = household_events.clone();

    let service = RestoreInventoryItemService::new(
        household_access_policy,
        inventory_item_repository.clone(),
        household_events_publisher.clone(),
    );

    (service, inventory_item_repository, household_repository)
}
