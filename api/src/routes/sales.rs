// Ventes — delegated to the shared gescom router (see gescom.rs)
// This stub is kept for historic compatibility. Actual routes are in gescom.rs.
use axum::Router;

pub fn router() -> Router<std::sync::Arc<crate::AppState>> {
    Router::new()
}
