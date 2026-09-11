use std::{convert::Infallible, time::Duration};

use axum::{
    extract::State,
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
};
use futures::{Stream, stream};

use crate::{
    modules::accounts::{
        api::CurrentUser, application::SubscribeUserEventsCommand, domain::UserEvent,
        ports::UserEventReceiverError,
    },
    shared::api::{ApiError, AppState},
};

pub async fn subscribe_user_events(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let command = SubscribeUserEventsCommand {
        requester_id: current_user.user_id(),
    };

    let receiver = state
        .accounts
        .subscribe_user_events
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    let shutdown = state.shutdown.clone();

    let stream = stream::unfold(
        (receiver, shutdown),
        |(mut receiver, shutdown)| async move {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    None
                }

                result = receiver.receive() => {
                    match result {
                        Ok(event) => {
                            let event = match event {
                                UserEvent::HouseholdMembershipChanged => {
                                    Event::default()
                                        .event("household_memberships_changed")
                                        .data("{}")
                                }
                            };

                            Some((
                                Ok::<_, Infallible>(event),
                                (receiver, shutdown),
                            ))
                        }

                        Err(UserEventReceiverError::Lagged) => {
                            let event = Event::default()
                                .event("household_memberships_changed")
                                .data("{}");

                            Some((
                                Ok::<_, Infallible>(event),
                                (receiver, shutdown),
                            ))
                        }

                        Err(UserEventReceiverError::Closed) => None,
                    }
                }
            }
        },
    );

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep_alive"),
    ))
}
