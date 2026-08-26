use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use crate::{
    modules::shopping::api::handlers::{
        create_custom_shopping_entry, delete_custom_shopping_entry, dismiss_shopping_item,
        list_shopping, set_custom_shopping_entry_checked, set_shopping_checked, set_shopping_note,
        set_shopping_quantity, update_custom_shopping_entry,
    },
    shared::api::AppState,
};

pub fn shopping_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_shopping))
        .route("/items/{item_id}/quantity", patch(set_shopping_quantity))
        .route("/items/{item_id}/note", patch(set_shopping_note))
        .route("/items/{item_id}/checked", patch(set_shopping_checked))
        .route("/items/{item_id}", delete(dismiss_shopping_item))
        .route("/custom/", post(create_custom_shopping_entry))
        .route(
            "/custom/{entry_id}",
            patch(update_custom_shopping_entry).delete(delete_custom_shopping_entry),
        )
        .route(
            "/custom/{entry_id}/checked",
            patch(set_custom_shopping_entry_checked),
        )
}
