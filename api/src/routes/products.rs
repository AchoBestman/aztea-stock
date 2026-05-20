use axum::{
    routing::{get, post, put, delete},
    Router,
};
use crate::controllers::product_controller::{
    create_product, delete_product, get_product, list_products, update_product,
};
use std::sync::Arc;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_products))
        .route("/", post(create_product))
        .route("/:id", get(get_product))
        .route("/:id", put(update_product))
        .route("/:id", delete(delete_product))
}
