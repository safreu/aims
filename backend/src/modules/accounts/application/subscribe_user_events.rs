use std::sync::Arc;

use crate::{
    modules::{
        accounts::ports::UserEventReceiver,
        accounts::{domain::UserId, ports::UserEventSubscriber},
    },
    shared::application::InternalError,
};

pub struct SubscribeUserEventsCommand {
    pub requester_id: UserId,
}

pub struct SubscribeUserEventsService {
    user_events_subscriber: Arc<dyn UserEventSubscriber>,
}

impl SubscribeUserEventsService {
    pub fn new(user_events_subscriber: Arc<dyn UserEventSubscriber>) -> Self {
        Self {
            user_events_subscriber,
        }
    }

    pub async fn execute(
        &self,
        command: SubscribeUserEventsCommand,
    ) -> Result<Box<dyn UserEventReceiver>, SubscribeUserEventsError> {
        let receiver = self
            .user_events_subscriber
            .subscribe(command.requester_id)
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    requester_id = %command.requester_id,
                    "Failed to subscribe to user events",
                );
                SubscribeUserEventsError::Internal(InternalError::Failed)
            })?;

        Ok(receiver)
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SubscribeUserEventsError {
    #[error(transparent)]
    Internal(#[from] InternalError),
}
