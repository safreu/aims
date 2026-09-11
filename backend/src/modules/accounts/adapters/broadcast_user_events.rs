use std::{collections::HashMap, sync::RwLock};

use tokio::sync::broadcast;

use crate::modules::{
    accounts::{
        adapters::broadcast_user_event_receiver::BroadcastUserEventReceiver,
        domain::UserId,
        ports::{UserEventPublisherError, UserEventSubscriber, UserEventSubscriberError},
    },
    accounts::{
        domain::UserEvent,
        ports::{UserEventPublisher, UserEventReceiver},
    },
};

pub struct BroadcastUserEvents {
    channels: RwLock<HashMap<UserId, broadcast::Sender<UserEvent>>>,
    capacity: usize,
}

impl BroadcastUserEvents {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Broadcast capacity must be greater than zero");

        Self {
            channels: RwLock::new(HashMap::new()),
            capacity,
        }
    }

    fn sender_for(
        &self,
        user_id: UserId,
    ) -> Result<broadcast::Sender<UserEvent>, USerEventChannelError> {
        let mut channels = self
            .channels
            .write()
            .map_err(|_| USerEventChannelError::Unavailable)?;

        let sender = channels
            .entry(user_id)
            .or_insert_with(|| {
                let (sender, _) = broadcast::channel(self.capacity);
                sender
            })
            .clone();

        Ok(sender)
    }
}

impl UserEventPublisher for BroadcastUserEvents {
    fn publish(&self, user_id: UserId, event: UserEvent) -> Result<(), UserEventPublisherError> {
        let sender = self
            .sender_for(user_id)
            .map_err(|_| UserEventPublisherError::Unavailable)?;

        let _ = sender.send(event);

        Ok(())
    }
}

impl UserEventSubscriber for BroadcastUserEvents {
    fn subscribe(
        &self,
        user_id: UserId,
    ) -> Result<Box<dyn UserEventReceiver>, UserEventSubscriberError> {
        let sender = self
            .sender_for(user_id)
            .map_err(|_| UserEventSubscriberError::Unavailable)?;

        let receiver = sender.subscribe();

        Ok(Box::new(BroadcastUserEventReceiver::new(receiver)))
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
enum USerEventChannelError {
    #[error("User event channel lock is unavailable")]
    Unavailable,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn subscriber_receives_published_event() {
        let events = BroadcastUserEvents::new(16);
        let user_id = UserId::new();

        let mut receiver = events
            .subscribe(user_id)
            .expect("Subscription should succeed");

        events
            .publish(user_id, UserEvent::HouseholdMembershipChanged)
            .expect("Publishing should succeed");

        let event = receiver.receive().await.expect("Receiving should succeed");

        assert_eq!(event, UserEvent::HouseholdMembershipChanged)
    }

    #[tokio::test]
    async fn multiple_subscribers_receive_same_event() {
        let events = BroadcastUserEvents::new(16);
        let user_id = UserId::new();

        let mut receiver = events
            .subscribe(user_id)
            .expect("Subscription should succeed");
        let mut another_receiver = events
            .subscribe(user_id)
            .expect("Subscription should succeed");

        events
            .publish(user_id, UserEvent::HouseholdMembershipChanged)
            .expect("Publishing should succeed");

        let event = receiver.receive().await.expect("Receiving should succeed");
        let another_event = another_receiver
            .receive()
            .await
            .expect("Receiving should succeed");

        assert_eq!(event, UserEvent::HouseholdMembershipChanged);
        assert_eq!(another_event, UserEvent::HouseholdMembershipChanged);
    }

    #[tokio::test]
    async fn subscribers_only_receive_events_for_their_household() {
        use std::time::Duration;

        let events = BroadcastUserEvents::new(16);
        let user_id = UserId::new();
        let another_user_id = UserId::new();

        let mut receiver = events
            .subscribe(user_id)
            .expect("Subscription should succeed");

        events
            .publish(another_user_id, UserEvent::HouseholdMembershipChanged)
            .expect("Publishing should succeed");

        let result = tokio::time::timeout(Duration::from_millis(50), receiver.receive()).await;

        assert!(result.is_err());

        events
            .publish(user_id, UserEvent::HouseholdMembershipChanged)
            .expect("Publishing should succeed");

        let event = receiver.receive().await.expect("Receiving should succeed");

        assert_eq!(event, UserEvent::HouseholdMembershipChanged)
    }

    #[tokio::test]
    async fn publishing_without_subscribers_succeeds() {
        let events = BroadcastUserEvents::new(16);
        let user_id = UserId::new();

        let result = events.publish(user_id, UserEvent::HouseholdMembershipChanged);

        assert!(result.is_ok())
    }
}
