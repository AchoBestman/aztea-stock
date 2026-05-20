use axum::{
    routing::{get, post, put, delete},
    Router,
};
use crate::controllers::stock_controller::{
    create_stock_item, list_stock_items, get_stock_item, update_stock_item, delete_stock_item,
    create_stock_movement, list_stock_movements,
};
use std::sync::Arc;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/items", get(list_stock_items))
        .route("/items", post(create_stock_item))
        .route("/items/:id", get(get_stock_item))
        .route("/items/:id", put(update_stock_item))
        .route("/items/:id", delete(delete_stock_item))
        .route("/movements", get(list_stock_movements))
        .route("/movements", post(create_stock_movement))
}
