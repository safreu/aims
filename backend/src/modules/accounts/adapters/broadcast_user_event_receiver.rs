use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::modules::accounts::{
    domain::UserEvent,
    ports::{UserEventReceiver, UserEventReceiverError},
};

pub struct BroadcastUserEventReceiver {
    receiver: broadcast::Receiver<UserEvent>,
}

impl BroadcastUserEventReceiver {
    pub fn new(receiver: broadcast::Receiver<UserEvent>) -> Self {
        Self { receiver }
    }
}

#[async_trait]
impl UserEventReceiver for BroadcastUserEventReceiver {
    async fn receive(&mut self) -> Result<UserEvent, UserEventReceiverError> {
        self.receiver.recv().await.map_err(|error| match error {
            broadcast::error::RecvError::Closed => UserEventReceiverError::Closed,
            broadcast::error::RecvError::Lagged(_) => UserEventReceiverError::Lagged,
        })
    }
}
