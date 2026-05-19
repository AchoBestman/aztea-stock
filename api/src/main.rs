use axum::Router;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::compression::CompressionLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod db;
mod errors;
mod middleware;
mod models;
mod routes;

pub struct AppState {
    pub db: Option<sqlx::AnyPool>,
    pub config: config::Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    // Initialize tracing safely with EnvFilter
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,api=debug")),
        )
        .init();

    let config = config::Config::from_env()?;
    let db = db::create_pool(&config).await;

    let state = Arc::new(AppState { db, config });

    let app = create_app(state.clone());

    let port = state.config.port;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("API AzteaStock démarrée sur :{}", port);
    axum::serve(listener, app).await?;
    Ok(())
}

pub fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", routes::ApiDoc::openapi()))
        .nest("/api/v1/health", routes::health::router())
        .nest("/api/v1/auth", routes::auth::router())
        .nest("/api/v1/products", routes::products::router())
        .nest("/api/v1/sales", routes::sales::router())
        .nest("/api/v1/stock", routes::stock::router())
        .nest("/api/v1/sync", routes::sync::router())
        .nest("/api/v1/reports", routes::reports::router())
        .nest("/api/v1/subscriptions", routes::subscriptions::router())
        .nest("/api/v1/admin", routes::admin::router())
        .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::auth::extract_tenant))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(CompressionLayer::new())
        .with_state(state)
}

#[cfg(test)]
mod tests;
