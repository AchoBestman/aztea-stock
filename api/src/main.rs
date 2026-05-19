use axum::Router;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::compression::CompressionLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod config;
pub mod database;
pub mod errors;
pub mod middleware;
pub mod models;
pub mod controllers;
pub mod routes;
pub mod repositories;
pub mod services;
pub mod dtos;
pub mod schemas;
pub mod utils;

pub struct AppState {
    pub db: Option<sea_orm::DatabaseConnection>,
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
    let db = database::create_connection(&config).await;

    let state = Arc::new(AppState { db, config });

    // Start background email queue worker
    services::email_service::start_email_worker(state.clone());

    // Start background license validity check (every 12 hours)
    middleware::license_guard::start_license_check_worker(state.clone());

    let app = create_app(state.clone());

    let port = state.config.port;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("API AzteaStock démarrée sur :{}", port);
    axum::serve(listener, app).await?;
    Ok(())
}

pub fn create_app(state: Arc<AppState>) -> Router {
    // ── Routes requiring only JWT (auth + license public actions) ────────────
    let jwt_only = Router::new()
        .nest("/api/v1/health", routes::health::router())
        .nest("/api/v1/auth", routes::auth::router())
        // License status & activate — auth required but no license guard (chicken-and-egg)
        .nest("/api/v1/licenses", routes::licenses::public_router())
        .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::auth::extract_tenant));

    // ── Business routes requiring JWT + active license ────────────────────────
    let licensed = Router::new()
        .nest("/api/v1/products", routes::products::router())
        .nest("/api/v1/sales", routes::sales::router())
        .nest("/api/v1/stock", routes::stock::router())
        .nest("/api/v1/sync", routes::sync::router())
        .nest("/api/v1/reports", routes::reports::router())
        .nest("/api/v1/admin", routes::role_routes::router()
            .merge(routes::tenant_routes::router())
            .merge(routes::user_routes::router())
            .merge(routes::subscriptions::admin_router())
            .merge(routes::licenses::admin_router()))
        // Apply JWT first, then license guard
        .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::license_guard::check_license))
        .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::auth::extract_tenant));

    let protected = jwt_only.merge(licensed);

    // ── Internal routes — NOT under JWT middleware (secured by x-internal-secret) ──
    let internal = Router::new()
        .nest("/api/v1/internal", routes::internal::router());

    Router::new()
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", routes::ApiDoc::openapi())
                .config(utoipa_swagger_ui::Config::default().persist_authorization(true))
        )
        .merge(protected)
        .merge(internal)
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(CompressionLayer::new())
        .with_state(state)
}

#[cfg(test)]
mod tests;
