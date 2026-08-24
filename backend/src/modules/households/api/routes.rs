use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::{
    modules::{
        devices::api::routes::device_routes,
        households::api::handlers::{
            add_household_member, create_household, get_household, list_household_members,
            list_households, remove_household_member, rename_household,
        },
    },
    shared::api::AppState,
};

pub fn households_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_household).get(list_households))
        .route("/{id}", get(get_household).patch(rename_household))
        .route(
            "/{id}/members",
            post(add_household_member).get(list_household_members),
        )
        .route("/{id}/members/{member_id}", delete(remove_household_member))
        .nest("/{household_id}/devices", device_routes())
}
