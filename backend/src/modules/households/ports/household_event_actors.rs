use async_trait::async_trait;

use crate::modules::households::domain::{HouseholdEvent, HouseholdId};

pub trait HouseholdEventPublisher: Send + Sync {
    fn publish(
        &self,
        household_id: HouseholdId,
        event: HouseholdEvent,
    ) -> Result<(), HouseholdEventPublisherError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HouseholdEventPublisherError {
    #[error("Household event publisher unavailable")]
    Unavailable,
}

pub trait HouseholdEventSubscriber: Send + Sync {
    fn subscribe(
        &self,
        household_id: HouseholdId,
    ) -> Result<Box<dyn HouseholdEventReceiver>, HouseholdEventSubscriberError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HouseholdEventSubscriberError {
    #[error("Household event subscriber unavailable")]
    Unavailable,
}

#[async_trait]
pub trait HouseholdEventReceiver: Send {
    async fn receive(&mut self) -> Result<HouseholdEvent, HouseholdEventReceiverError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HouseholdEventReceiverError {
    #[error("event stream closed")]
    Closed,
    #[error("event stream lagged")]
    Lagged,
}
