use axum::{
    routing::{get, post},
    Router
};
use std::sync::Arc;
use crate::{AppState, controllers::tenant_controller};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/tenant", get(tenant_controller::get_tenant).put(tenant_controller::update_tenant))
        .route("/tenant/two-factor", post(tenant_controller::set_tenant_two_factor))
        .route("/tenants", get(tenant_controller::list_tenants).post(tenant_controller::create_tenant))
        .route(
            "/tenants/:id",
            get(tenant_controller::get_tenant_by_id).delete(tenant_controller::delete_tenant),
        )
}
