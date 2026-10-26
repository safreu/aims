use std::sync::Arc;

use crate::{
    modules::{
        accounts::{
            domain::{UserEvent, UserId},
            ports::UserEventPublisher,
        },
        households::{
            domain::{HouseholdApplicationEvent, HouseholdEvent, HouseholdId},
            ports::HouseholdEventPublisher,
        },
        inventory::domain::InventoryEvent,
    },
    shared::application::events::{AppEvent, EventDispatcher},
};

pub struct DefaultEventDispatcher {
    household_events: Arc<dyn HouseholdEventPublisher>,
    user_events: Arc<dyn UserEventPublisher>,
}

impl DefaultEventDispatcher {
    pub fn new(
        household_events: Arc<dyn HouseholdEventPublisher>,
        user_events: Arc<dyn UserEventPublisher>,
    ) -> Self {
        Self {
            household_events,
            user_events,
        }
    }

    fn dispatch_inventory_event(&self, event: InventoryEvent) {
        match event {
            InventoryEvent::StockChanged { household_id, .. }
            | InventoryEvent::ItemCreated { household_id, .. }
            | InventoryEvent::ItemUpdated { household_id, .. }
            | InventoryEvent::ItemArchived { household_id, .. }
            | InventoryEvent::ItemRestored { household_id, .. } => {
                self.publish_household_event(household_id, HouseholdEvent::ShoppingListChanged);
                self.publish_household_event(household_id, HouseholdEvent::InventoryItemsChanged);
            }

            InventoryEvent::CategoryCreated { household_id, .. }
            | InventoryEvent::CategoryDeleted { household_id, .. }
            | InventoryEvent::CategoryUpdated { household_id, .. } => {
                self.publish_household_event(household_id, HouseholdEvent::ShoppingListChanged);
                self.publish_household_event(household_id, HouseholdEvent::InventoryItemsChanged);
                self.publish_household_event(
                    household_id,
                    HouseholdEvent::InventoryCategoriesChanged,
                );
            }
        }
    }

    fn dispatch_household_event(&self, event: HouseholdApplicationEvent) {
        match event {
            HouseholdApplicationEvent::Created {
                household_id: _,
                owner_id,
            } => {
                self.publish_user_event(owner_id, UserEvent::HouseholdMembershipChanged);
            }
            HouseholdApplicationEvent::NameUpdated { household_id } => {
                self.publish_household_event(household_id, HouseholdEvent::HouseholdChanged);
            }
            HouseholdApplicationEvent::Deleted {
                household_id: _,
                owner_id,
            } => {
                self.publish_user_event(owner_id, UserEvent::HouseholdMembershipChanged);
            }
            HouseholdApplicationEvent::MemberAdded {
                household_id,
                member_id,
            }
            | HouseholdApplicationEvent::MemberLeft {
                household_id,
                member_id,
            }
            | HouseholdApplicationEvent::MemberRemoved {
                household_id,
                member_id,
            } => {
                self.publish_household_event(household_id, HouseholdEvent::HouseholdChanged);
                self.publish_user_event(member_id, UserEvent::HouseholdMembershipChanged);
            }
        }
    }

    fn publish_household_event(&self, household_id: HouseholdId, event: HouseholdEvent) {
        if let Err(error) = self.household_events.publish(household_id, event.clone()) {
            tracing::error!(
                error = ?error,
                household_id = %household_id,
                event = ?event,
                "Failed to publish household event"
            )
        }
    }

    fn publish_user_event(&self, user_id: UserId, event: UserEvent) {
        if let Err(error) = self.user_events.publish(user_id, event.clone()) {
            tracing::error!(
                error = ?error,
                user_id = %user_id,
                event = ?event,
                "Failed to publish user event"
            )
        }
    }
}

impl EventDispatcher for DefaultEventDispatcher {
    fn dispatch(&self, event: AppEvent) {
        match event {
            AppEvent::Inventory(event) => {
                self.dispatch_inventory_event(event);
            }
            AppEvent::Household(event) => {
                self.dispatch_household_event(event);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crate::modules::{
        accounts::{
            domain::{UserEvent, UserId},
            ports::{UserEventPublisher, UserEventPublisherError},
        },
        households::{
            domain::{HouseholdApplicationEvent, HouseholdEvent, HouseholdId},
            ports::{HouseholdEventPublisher, HouseholdEventPublisherError},
        },
        inventory::domain::{CategoryId, InventoryEvent, InventoryItemId},
    };

    use super::*;

    struct RecordingHouseholdEventPublisher {
        published: Mutex<Vec<(HouseholdId, HouseholdEvent)>>,
    }

    impl RecordingHouseholdEventPublisher {
        fn new() -> Self {
            Self {
                published: Mutex::new(Vec::new()),
            }
        }

        fn published(&self) -> Vec<(HouseholdId, HouseholdEvent)> {
            self.published
                .lock()
                .expect("Published household events lock should not be poisoned")
                .clone()
        }
    }

    impl HouseholdEventPublisher for RecordingHouseholdEventPublisher {
        fn publish(
            &self,
            household_id: HouseholdId,
            event: HouseholdEvent,
        ) -> Result<(), HouseholdEventPublisherError> {
            self.published
                .lock()
                .expect("Published household events lock should not be poisoned")
                .push((household_id, event));

            Ok(())
        }
    }

    struct RecordingUserEventPublisher {
        published: Mutex<Vec<(UserId, UserEvent)>>,
    }

    impl RecordingUserEventPublisher {
        fn new() -> Self {
            Self {
                published: Mutex::new(Vec::new()),
            }
        }

        fn published(&self) -> Vec<(UserId, UserEvent)> {
            self.published
                .lock()
                .expect("Published user events lock should not be poisoned")
                .clone()
        }
    }

    impl UserEventPublisher for RecordingUserEventPublisher {
        fn publish(
            &self,
            user_id: UserId,
            event: UserEvent,
        ) -> Result<(), UserEventPublisherError> {
            self.published
                .lock()
                .expect("Published user events lock should not be poisoned")
                .push((user_id, event));

            Ok(())
        }
    }

    fn build_dispatcher() -> (
        DefaultEventDispatcher,
        Arc<RecordingHouseholdEventPublisher>,
        Arc<RecordingUserEventPublisher>,
    ) {
        let household_publisher = Arc::new(RecordingHouseholdEventPublisher::new());

        let user_publisher = Arc::new(RecordingUserEventPublisher::new());

        let dispatcher =
            DefaultEventDispatcher::new(household_publisher.clone(), user_publisher.clone());

        (dispatcher, household_publisher, user_publisher)
    }

    fn assert_item_event_dispatch(event: InventoryEvent, household_id: HouseholdId) {
        let (dispatcher, household_publisher, user_publisher) = build_dispatcher();

        dispatcher.dispatch(AppEvent::Inventory(event));

        assert_eq!(
            household_publisher.published(),
            vec![
                (household_id, HouseholdEvent::ShoppingListChanged,),
                (household_id, HouseholdEvent::InventoryItemsChanged,),
            ]
        );

        assert!(user_publisher.published().is_empty());
    }

    fn assert_category_event_dispatch(event: InventoryEvent, household_id: HouseholdId) {
        let (dispatcher, household_publisher, user_publisher) = build_dispatcher();

        dispatcher.dispatch(AppEvent::Inventory(event));

        assert_eq!(
            household_publisher.published(),
            vec![
                (household_id, HouseholdEvent::ShoppingListChanged,),
                (household_id, HouseholdEvent::InventoryItemsChanged,),
                (household_id, HouseholdEvent::InventoryCategoriesChanged,),
            ]
        );

        assert!(user_publisher.published().is_empty());
    }

    #[test]
    fn stock_changed_publishes_shopping_list_and_inventory_items_changed() {
        let household_id = HouseholdId::new();

        assert_item_event_dispatch(
            InventoryEvent::StockChanged {
                household_id,
                item_id: InventoryItemId::new(),
            },
            household_id,
        );
    }

    #[test]
    fn item_created_publishes_shopping_list_and_inventory_items_changed() {
        let household_id = HouseholdId::new();

        assert_item_event_dispatch(
            InventoryEvent::ItemCreated {
                household_id,
                item_id: InventoryItemId::new(),
            },
            household_id,
        );
    }

    #[test]
    fn item_updated_publishes_shopping_list_and_inventory_items_changed() {
        let household_id = HouseholdId::new();

        assert_item_event_dispatch(
            InventoryEvent::ItemUpdated {
                household_id,
                item_id: InventoryItemId::new(),
            },
            household_id,
        );
    }

    #[test]
    fn item_archived_publishes_shopping_list_and_inventory_items_changed() {
        let household_id = HouseholdId::new();

        assert_item_event_dispatch(
            InventoryEvent::ItemArchived {
                household_id,
                item_id: InventoryItemId::new(),
            },
            household_id,
        );
    }

    #[test]
    fn item_restored_publishes_shopping_list_and_inventory_items_changed() {
        let household_id = HouseholdId::new();

        assert_item_event_dispatch(
            InventoryEvent::ItemRestored {
                household_id,
                item_id: InventoryItemId::new(),
            },
            household_id,
        );
    }

    #[test]
    fn category_created_publishes_all_relevant_household_events() {
        let household_id = HouseholdId::new();

        assert_category_event_dispatch(
            InventoryEvent::CategoryCreated {
                household_id,
                category_id: CategoryId::new(),
            },
            household_id,
        );
    }

    #[test]
    fn category_updated_publishes_all_relevant_household_events() {
        let household_id = HouseholdId::new();

        assert_category_event_dispatch(
            InventoryEvent::CategoryUpdated {
                household_id,
                category_id: CategoryId::new(),
            },
            household_id,
        );
    }

    #[test]
    fn category_deleted_publishes_all_relevant_household_events() {
        let household_id = HouseholdId::new();

        assert_category_event_dispatch(
            InventoryEvent::CategoryDeleted {
                household_id,
                category_id: CategoryId::new(),
            },
            household_id,
        );
    }

    #[test]
    fn household_created_publishes_membership_changed_for_owner() {
        let (dispatcher, household_publisher, user_publisher) = build_dispatcher();

        let household_id = HouseholdId::new();
        let owner_id = UserId::new();

        dispatcher.dispatch(AppEvent::Household(HouseholdApplicationEvent::Created {
            household_id,
            owner_id,
        }));

        assert!(household_publisher.published().is_empty());

        assert_eq!(
            user_publisher.published(),
            vec![(owner_id, UserEvent::HouseholdMembershipChanged,)]
        );
    }

    #[test]
    fn household_name_updated_publishes_household_changed() {
        let (dispatcher, household_publisher, user_publisher) = build_dispatcher();

        let household_id = HouseholdId::new();

        dispatcher.dispatch(AppEvent::Household(
            HouseholdApplicationEvent::NameUpdated { household_id },
        ));

        assert_eq!(
            household_publisher.published(),
            vec![(household_id, HouseholdEvent::HouseholdChanged,)]
        );

        assert!(user_publisher.published().is_empty());
    }

    #[test]
    fn household_deleted_publishes_membership_changed_for_owner() {
        let (dispatcher, household_publisher, user_publisher) = build_dispatcher();

        let household_id = HouseholdId::new();
        let owner_id = UserId::new();

        dispatcher.dispatch(AppEvent::Household(HouseholdApplicationEvent::Deleted {
            household_id,
            owner_id,
        }));

        assert!(household_publisher.published().is_empty());

        assert_eq!(
            user_publisher.published(),
            vec![(owner_id, UserEvent::HouseholdMembershipChanged,)]
        );
    }

    #[test]
    fn member_added_publishes_household_and_membership_changed() {
        let (dispatcher, household_publisher, user_publisher) = build_dispatcher();

        let household_id = HouseholdId::new();
        let member_id = UserId::new();

        dispatcher.dispatch(AppEvent::Household(
            HouseholdApplicationEvent::MemberAdded {
                household_id,
                member_id,
            },
        ));

        assert_eq!(
            household_publisher.published(),
            vec![(household_id, HouseholdEvent::HouseholdChanged,)]
        );

        assert_eq!(
            user_publisher.published(),
            vec![(member_id, UserEvent::HouseholdMembershipChanged,)]
        );
    }

    #[test]
    fn member_left_publishes_household_and_membership_changed() {
        let (dispatcher, household_publisher, user_publisher) = build_dispatcher();

        let household_id = HouseholdId::new();
        let member_id = UserId::new();

        dispatcher.dispatch(AppEvent::Household(HouseholdApplicationEvent::MemberLeft {
            household_id,
            member_id,
        }));

        assert_eq!(
            household_publisher.published(),
            vec![(household_id, HouseholdEvent::HouseholdChanged,)]
        );

        assert_eq!(
            user_publisher.published(),
            vec![(member_id, UserEvent::HouseholdMembershipChanged,)]
        );
    }

    #[test]
    fn member_removed_publishes_household_and_membership_changed() {
        let (dispatcher, household_publisher, user_publisher) = build_dispatcher();

        let household_id = HouseholdId::new();
        let member_id = UserId::new();

        dispatcher.dispatch(AppEvent::Household(
            HouseholdApplicationEvent::MemberRemoved {
                household_id,
                member_id,
            },
        ));

        assert_eq!(
            household_publisher.published(),
            vec![(household_id, HouseholdEvent::HouseholdChanged,)]
        );

        assert_eq!(
            user_publisher.published(),
            vec![(member_id, UserEvent::HouseholdMembershipChanged,)]
        );
    }
}
