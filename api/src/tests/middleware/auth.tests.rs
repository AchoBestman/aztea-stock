use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
    Json,
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use std::sync::Arc;
use jsonwebtoken::{encode, Header, EncodingKey};
use serde_json::Value;

use crate::{
    AppState,
    config::Config,
    middleware::auth::{extract_tenant, Claims},
};

// Helper to generate a test JWT token
fn generate_token(secret: &str, role: &str) -> String {
    let claims = Claims {
        sub: "user_123".to_string(),
        tenant_id: "tenant_456".to_string(),
        role: role.to_string(),
        exp: (chrono::Utc::now().timestamp() + 3600) as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

// Setup a test router containing the extract_tenant middleware
fn setup_auth_test_app(secret: String) -> Router {
    let config = Config {
        database_url: None,
        sqlite_database_url: "sqlite://:memory:".to_string(),
        db_type: "postgres".to_string(),
        offline: false,
        jwt_secret: secret,
        port: 8080,
        rust_log: "info".to_string(),
        ..Config::default()
    };
    let state = Arc::new(AppState { db: None, config });

    Router::new()
        // A mock private endpoint to test auth
        .route(
            "/api/v1/products",
            get(|req: Request<Body>| async move {
                // Verify the claims extensions are present
                let claims = req.extensions().get::<Claims>().cloned();
                if let Some(c) = claims {
                    axum::response::Json(serde_json::json!({
                        "success": true,
                        "user_id": c.sub,
                        "tenant_id": c.tenant_id
                    }))
                } else {
                    axum::response::Json(serde_json::json!({ "success": false }))
                }
            }),
        )
        // Public endpoint
        .route("/api/v1/auth/login", get(|| async { "login" }))
        .layer(axum::middleware::from_fn_with_state(state.clone(), extract_tenant))
        .with_state(state)
}

#[tokio::test]
async fn test_public_route_bypass() {
    let app = setup_auth_test_app("secret".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_missing_auth_header() {
    let app = setup_auth_test_app("secret".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/products")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_malformed_auth_header() {
    let app = setup_auth_test_app("secret".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/products")
                .header("Authorization", "MalformedTokenString")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_invalid_jwt_signature() {
    let app = setup_auth_test_app("secret".to_string());
    // Token signed with a different secret
    let token = generate_token("wrong_secret", "manager");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/products")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_successful_auth() {
    let secret = "secret_key_12345_secret_key_12345".to_string();
    let app = setup_auth_test_app(secret.clone());
    let token = generate_token(&secret, "manager");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/products")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body();
    let bytes = body.collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(body["success"], true);
    assert_eq!(body["user_id"], "user_123");
    assert_eq!(body["tenant_id"], "tenant_456");
}
