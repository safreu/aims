use async_trait::async_trait;

use crate::modules::accounts::domain::{UserEvent, UserId};

pub trait UserEventPublisher: Send + Sync {
    fn publish(&self, user_id: UserId, event: UserEvent) -> Result<(), UserEventPublisherError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UserEventPublisherError {
    #[error("User event publisher unavailable")]
    Unavailable,
}

pub trait UserEventSubscriber: Send + Sync {
    fn subscribe(
        &self,
        user_id: UserId,
    ) -> Result<Box<dyn UserEventReceiver>, UserEventSubscriberError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UserEventSubscriberError {
    #[error("User event subscriber unavailable")]
    Unavailable,
}

#[async_trait]
pub trait UserEventReceiver: Send {
    async fn receive(&mut self) -> Result<UserEvent, UserEventReceiverError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UserEventReceiverError {
    #[error("event stream closed")]
    Closed,
    #[error("event stream lagged")]
    Lagged,
}
