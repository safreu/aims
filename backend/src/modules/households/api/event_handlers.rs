use std::{convert::Infallible, time::Duration};

use axum::{
    extract::{Path, State},
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
};
use futures::{Stream, stream};
use uuid::Uuid;

use crate::{
    modules::{
        accounts::api::CurrentUser,
        households::{
            application::SubscribeHouseholdEventsCommand,
            domain::{HouseholdEvent, HouseholdId},
            ports::HouseholdEventReceiverError,
        },
    },
    shared::api::{ApiError, AppState},
};

pub async fn subscribe_household_events(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let command = SubscribeHouseholdEventsCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
    };

    let receiver = state
        .households
        .subscribe_household_events
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    let stream = stream::unfold(receiver, |mut receiver| async move {
        match receiver.receive().await {
            Ok(event) => {
                let event = match event {
                    HouseholdEvent::ShoppingListChanged => {
                        Event::default().event("shopping_list_changed").data("{}")
                    }
                    HouseholdEvent::InventoryCategoriesChanged => Event::default()
                        .event("inventory_categories_changed")
                        .data("{}"),
                    HouseholdEvent::InventoryItemsChanged => {
                        Event::default().event("inventory_items_changed").data("{}")
                    }
                    HouseholdEvent::HouseholdChanged => {
                        Event::default().event("household_changed").data("{}")
                    }
                };
                Some((Ok::<_, Infallible>(event), receiver))
            }
            Err(HouseholdEventReceiverError::Lagged) => {
                let event = Event::default()
                    .event("household_resync_required")
                    .data("{}");

                Some((Ok::<_, Infallible>(event), receiver))
            }

            Err(HouseholdEventReceiverError::Closed) => None,
        }
    });

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep_alive"),
    ))
}
