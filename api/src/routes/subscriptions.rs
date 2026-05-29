use crate::AppState;
use crate::controllers::subscription_controller::{
    create_subscription, delete_subscription, list_subscriptions, update_subscription,
    update_subscription_status,
};
use axum::{
    Router,
    routing::{delete, patch, post},
};
use std::sync::Arc;

pub fn admin_router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/subscriptions",
            post(create_subscription).get(list_subscriptions),
        )
        .route(
            "/subscriptions/:id",
            delete(delete_subscription).put(update_subscription),
        )
        .route(
            "/subscriptions/:id/status",
            patch(update_subscription_status),
        )
}
