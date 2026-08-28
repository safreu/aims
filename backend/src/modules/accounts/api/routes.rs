use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    modules::accounts::api::handlers::{get_user, login_user, logout_user, register_user},
    shared::api::AppState,
};

pub fn accounts_router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .route("/logout", post(logout_user))
        .route("/me", get(get_user))
}
