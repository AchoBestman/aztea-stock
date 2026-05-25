use axum::{routing::{post, delete, patch}, Router};
use std::sync::Arc;
use crate::AppState;
use crate::controllers::subscription_controller::{create_subscription, list_subscriptions, delete_subscription, update_subscription_status};

pub fn admin_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/subscriptions", post(create_subscription).get(list_subscriptions))
        .route("/subscriptions/:id", delete(delete_subscription))
        .route("/subscriptions/:id/status", patch(update_subscription_status))
}
