use axum::{
    routing::{get, post},
    Router
};
use std::sync::Arc;
use crate::{AppState, controllers::user_controller};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users", get(user_controller::list_users).post(user_controller::create_user))
        .route("/users/two-factor", post(user_controller::set_user_two_factor))
        .route("/users/password", post(user_controller::set_user_password))
        .route("/users/send-reset", post(user_controller::send_user_reset))
}
