use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::{
    modules::inventory::api::handlers::{
        archive_inventory_item, create_category, create_inventory_item, decrease_inventory_stock,
        delete_category, get_inventory_item, increase_inventory_stock, list_categories,
        list_inventory_items, list_inventory_stock_history, restore_inventory_item,
        set_inventory_stock, update_inventory_item,
    },
    shared::api::AppState,
};

pub fn inventory_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/{household_id}/items",
            post(create_inventory_item).get(list_inventory_items),
        )
        .route(
            "/{household_id}/items/{item_id}",
            get(get_inventory_item).patch(update_inventory_item),
        )
        .route(
            "/{household_id}/items/{item_id}/archive",
            post(archive_inventory_item),
        )
        .route(
            "/{household_id}/items/{item_id}/restore",
            post(restore_inventory_item),
        )
        .route(
            "/{household_id}/items/{item_id}/increase",
            post(increase_inventory_stock),
        )
        .route(
            "/{household_id}/items/{item_id}/decrease",
            post(decrease_inventory_stock),
        )
        .route(
            "/{household_id}/items/{item_id}/stock",
            put(set_inventory_stock),
        )
        .route(
            "/{household_id}/items/{item_id}/history",
            get(list_inventory_stock_history),
        )
        .route(
            "/{household_id}/categories",
            post(create_category).get(list_categories),
        )
        .route(
            "/{household_id}/categories/{category_id}",
            delete(delete_category),
        )
}
