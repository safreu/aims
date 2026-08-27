use std::{collections::HashMap, sync::RwLock};

use tokio::sync::broadcast;

use crate::modules::households::{
    adapters::BroadcastHouseholdEventReceiver,
    domain::{HouseholdEvent, HouseholdId},
    ports::{
        HouseholdEventPublisher, HouseholdEventPublisherError, HouseholdEventReceiver,
        HouseholdEventSubscriber, HouseholdEventSubscriberError,
    },
};

pub struct BroadcastHouseholdEvents {
    channels: RwLock<HashMap<HouseholdId, broadcast::Sender<HouseholdEvent>>>,
    capacity: usize,
}

impl BroadcastHouseholdEvents {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Broadcast capacity must be greater than zero");

        Self {
            channels: RwLock::new(HashMap::new()),
            capacity,
        }
    }

    fn sender_for(
        &self,
        household_id: HouseholdId,
    ) -> Result<broadcast::Sender<HouseholdEvent>, HouseholdEventChannelError> {
        let mut channels = self
            .channels
            .write()
            .map_err(|_| HouseholdEventChannelError::Unavailable)?;

        let sender = channels
            .entry(household_id)
            .or_insert_with(|| {
                let (sender, _) = broadcast::channel(self.capacity);
                sender
            })
            .clone();

        Ok(sender)
    }
}

impl HouseholdEventPublisher for BroadcastHouseholdEvents {
    fn publish(
        &self,
        household_id: HouseholdId,
        event: HouseholdEvent,
    ) -> Result<(), HouseholdEventPublisherError> {
        let sender = self
            .sender_for(household_id)
            .map_err(|_| HouseholdEventPublisherError::Unavailable)?;

        let _ = sender.send(event);

        Ok(())
    }
}

impl HouseholdEventSubscriber for BroadcastHouseholdEvents {
    fn subscribe(
        &self,
        household_id: HouseholdId,
    ) -> Result<Box<dyn HouseholdEventReceiver>, HouseholdEventSubscriberError> {
        let sender = self
            .sender_for(household_id)
            .map_err(|_| HouseholdEventSubscriberError::Unavailable)?;

        let receiver = sender.subscribe();

        Ok(Box::new(BroadcastHouseholdEventReceiver::new(receiver)))
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
enum HouseholdEventChannelError {
    #[error("Household event channel lock is unavailable")]
    Unavailable,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn subscriber_receives_published_event() {
        let events = BroadcastHouseholdEvents::new(16);
        let household_id = HouseholdId::new();

        let mut receiver = events
            .subscribe(household_id)
            .expect("Subscription should succeed");

        events
            .publish(household_id, HouseholdEvent::ShoppingListChanged)
            .expect("Publishing should succeed");

        let event = receiver.receive().await.expect("Receiving should succeed");

        assert_eq!(event, HouseholdEvent::ShoppingListChanged)
    }

    #[tokio::test]
    async fn multiple_subscribers_receive_same_event() {
        let events = BroadcastHouseholdEvents::new(16);
        let household_id = HouseholdId::new();

        let mut receiver = events
            .subscribe(household_id)
            .expect("Subscription should succeed");
        let mut another_receiver = events
            .subscribe(household_id)
            .expect("Subscription should succeed");

        events
            .publish(household_id, HouseholdEvent::ShoppingListChanged)
            .expect("Publishing should succeed");

        let event = receiver.receive().await.expect("Receiving should succeed");
        let another_event = another_receiver
            .receive()
            .await
            .expect("Receiving should succeed");

        assert_eq!(event, HouseholdEvent::ShoppingListChanged);
        assert_eq!(another_event, HouseholdEvent::ShoppingListChanged);
    }

    #[tokio::test]
    async fn subscribers_only_receive_events_for_their_household() {
        use std::time::Duration;

        let events = BroadcastHouseholdEvents::new(16);
        let household_id = HouseholdId::new();
        let another_household_id = HouseholdId::new();

        let mut receiver = events
            .subscribe(household_id)
            .expect("Subscription should succeed");

        events
            .publish(another_household_id, HouseholdEvent::ShoppingListChanged)
            .expect("Publishing should succeed");

        let result = tokio::time::timeout(Duration::from_millis(50), receiver.receive()).await;

        assert!(result.is_err());

        events
            .publish(household_id, HouseholdEvent::ShoppingListChanged)
            .expect("Publishing should succeed");

        let event = receiver.receive().await.expect("Receiving should succeed");

        assert_eq!(event, HouseholdEvent::ShoppingListChanged)
    }

    #[tokio::test]
    async fn publishing_without_subscribers_succeeds() {
        let events = BroadcastHouseholdEvents::new(16);
        let household_id = HouseholdId::new();

        let result = events.publish(household_id, HouseholdEvent::ShoppingListChanged);

        assert!(result.is_ok())
    }
}
