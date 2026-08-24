use axum::{
    Router,
    routing::{patch, post},
};

use crate::{
    modules::devices::api::handlers::{
        list_devices, register_device, rename_device, revoke_device,
    },
    shared::api::AppState,
};

pub fn device_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(register_device).get(list_devices))
        .route("/{device_id}", patch(rename_device))
        .route("/{device_id}/revoke", post(revoke_device))
}
