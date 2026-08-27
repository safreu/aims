use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::modules::households::{
    domain::HouseholdEvent,
    ports::{HouseholdEventReceiver, HouseholdEventReceiverError},
};

pub struct BroadcastHouseholdEventReceiver {
    receiver: broadcast::Receiver<HouseholdEvent>,
}

impl BroadcastHouseholdEventReceiver {
    pub fn new(receiver: broadcast::Receiver<HouseholdEvent>) -> Self {
        Self { receiver }
    }
}

#[async_trait]
impl HouseholdEventReceiver for BroadcastHouseholdEventReceiver {
    async fn receive(&mut self) -> Result<HouseholdEvent, HouseholdEventReceiverError> {
        self.receiver.recv().await.map_err(|error| match error {
            broadcast::error::RecvError::Closed => HouseholdEventReceiverError::Closed,
            broadcast::error::RecvError::Lagged(_) => HouseholdEventReceiverError::Lagged,
        })
    }
}
