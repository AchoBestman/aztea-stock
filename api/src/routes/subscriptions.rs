use axum::{routing::post, Router};
use std::sync::Arc;
use crate::AppState;
use crate::controllers::subscription_controller::{create_subscription, list_subscriptions};

pub fn admin_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/subscriptions", post(create_subscription).get(list_subscriptions))
}
