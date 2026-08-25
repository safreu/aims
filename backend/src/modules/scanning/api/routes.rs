use axum::{Router, routing::post};

use crate::{
    modules::scanning::api::handlers::{
        create_qr_action, execute_qr_action, list_qr_actions, revoke_qr_action,
    },
    shared::api::AppState,
};

pub fn scanning_management_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_qr_action).get(list_qr_actions))
        .route("/{qr_action_id}/revoke", post(revoke_qr_action))
}

pub fn scanning_device_routes() -> Router<AppState> {
    Router::new().route("/qr/{qr_action_id}/execute", post(execute_qr_action))
}
