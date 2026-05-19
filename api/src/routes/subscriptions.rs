use axum::{routing::{post, get}, Router};
use std::sync::Arc;
use crate::AppState;
use crate::controllers::subscription_controller::{create_subscription, list_tenant_subscriptions};

pub fn admin_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/subscriptions", post(create_subscription))
        .route("/tenants/:tenant_id/subscriptions", get(list_tenant_subscriptions))
}
