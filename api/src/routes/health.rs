use axum::{routing::get, Json, Router};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    /// Service status indicators: "OK" or "DEGRADED"
    pub status: String,
    /// The API service semantic version
    pub version: String,
    /// Database connection status
    pub database_connected: bool,
}

#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses(
        (status = 200, description = "Retrieve API health status", body = HealthResponse)
    ),
    tag = "Health"
)]
pub async fn health_check(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<crate::AppState>>,
) -> Json<HealthResponse> {
    let db_connected = state.db.is_some();
    Json(HealthResponse {
        status: if db_connected { "OK".to_string() } else { "DEGRADED".to_string() },
        version: env!("CARGO_PKG_VERSION").to_string(),
        database_connected: db_connected,
    })
}

pub fn router() -> Router<std::sync::Arc<crate::AppState>> {
    Router::new().route("/", get(health_check))
}
