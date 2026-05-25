use axum::{routing::{post, get}, Router};
use std::sync::Arc;
use crate::AppState;
use crate::controllers::license_controller::{generate_license, list_licenses, activate_license, get_license_status, reveal_license_key, send_license_key_email};

pub fn admin_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/licenses", post(generate_license).get(list_licenses))
        .route("/licenses/:id/reveal", get(reveal_license_key))
        .route("/licenses/:id/send-key", post(send_license_key_email))
}

/// Routes under /api/v1/licenses — accessible to any authenticated user
pub fn public_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/activate", post(activate_license))
        .route("/status", get(get_license_status))
}

