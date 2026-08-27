use std::sync::Arc;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            domain::HouseholdId,
            ports::{
                HouseholdAccessError, HouseholdAccessPolicy, HouseholdEventReceiver,
                HouseholdEventSubscriber,
            },
        },
    },
    shared::application::InternalError,
};

pub struct SubscribeHouseholdEventsCommand {
    pub requester_id: UserId,
    pub household_id: HouseholdId,
}

pub struct SubscribeHouseholdEventsService {
    household_access_policy: Arc<dyn HouseholdAccessPolicy>,
    household_events_subscriber: Arc<dyn HouseholdEventSubscriber>,
}

impl SubscribeHouseholdEventsService {
    pub fn new(
        household_access_policy: Arc<dyn HouseholdAccessPolicy>,
        household_events_subscriber: Arc<dyn HouseholdEventSubscriber>,
    ) -> Self {
        Self {
            household_access_policy,
            household_events_subscriber,
        }
    }

    pub async fn execute(
        &self,
        command: SubscribeHouseholdEventsCommand,
    ) -> Result<Box<dyn HouseholdEventReceiver>, SubscribeHouseholdEventsError> {
        self.household_access_policy
            .require_member(&command.household_id, &command.requester_id)
            .await?;

        let receiver = self
            .household_events_subscriber
            .subscribe(command.household_id)
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    household_id = %command.household_id,
                    "Failed to subscribe to household events",
                );
                SubscribeHouseholdEventsError::Internal(InternalError::Failed)
            })?;

        Ok(receiver)
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SubscribeHouseholdEventsError {
    #[error(transparent)]
    HouseholdAccess(#[from] HouseholdAccessError),
    #[error(transparent)]
    Internal(#[from] InternalError),
}
