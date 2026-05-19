use axum::{
    routing::get,
    Router
};
use std::sync::Arc;
use crate::{AppState, controllers::role as role_controller};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/roles", get(role_controller::list_roles).post(role_controller::create_role))
        .route("/roles/:id", get(role_controller::get_role).put(role_controller::update_role).delete(role_controller::delete_role))
}
