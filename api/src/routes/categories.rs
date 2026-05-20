use axum::{
    routing::{get, post, put, delete},
    Router,
};
use crate::controllers::category_controller::{
    create_category, delete_category, get_category, list_categories, update_category,
};
use std::sync::Arc;
use crate::AppState;


pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_categories))
        .route("/", post(create_category))
        .route("/:id", get(get_category))
        .route("/:id", put(update_category))
        .route("/:id", delete(delete_category))
}
