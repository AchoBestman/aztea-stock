use axum::{routing::{post, get}, Router};
use std::sync::Arc;
use crate::AppState;
use crate::controllers::license_controller::{generate_license, list_tenant_licenses, activate_license};

pub fn admin_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/licenses", post(generate_license))
        .route("/tenants/:tenant_id/licenses", get(list_tenant_licenses))
}

pub fn public_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/licenses/activate", post(activate_license))
}
