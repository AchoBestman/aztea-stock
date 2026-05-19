use axum::Router;

pub fn router() -> Router<std::sync::Arc<crate::AppState>> {
    Router::new()
}
