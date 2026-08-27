use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::{
    modules::{
        devices::api::routes::device_routes,
        households::api::{
            event_handlers::subscribe_household_events,
            handlers::{
                add_household_member, create_household, get_household, list_household_members,
                list_households, remove_household_member, rename_household,
            },
        },
        scanning::api::scanning_management_routes,
        shopping::api::shopping_routes,
    },
    shared::api::AppState,
};

pub fn households_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_household).get(list_households))
        .route(
            "/{household_id}",
            get(get_household).patch(rename_household),
        )
        .route(
            "/{household_id}/members",
            post(add_household_member).get(list_household_members),
        )
        .route(
            "/{household_id}/members/{member_id}",
            delete(remove_household_member),
        )
        .route("/{household_id}/events", get(subscribe_household_events))
        .nest("/{household_id}/devices", device_routes())
        .nest("/{household_id}/qr", scanning_management_routes())
        .nest("/{household_id}/shopping", shopping_routes())
}
