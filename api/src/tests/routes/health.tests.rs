use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use std::sync::Arc;
use serde_json::Value;

use crate::{create_app, AppState, config::Config};

#[tokio::test]
async fn test_health_check_endpoint() {
    let config = Config {
        database_url: None,
        sqlite_database_url: "sqlite://:memory:".to_string(),
        db_type: "postgres".to_string(),
        offline: false,
        jwt_secret: "test_jwt_secret_123456_test_jwt_secret".to_string(),
        port: 8080,
        rust_log: "info".to_string(),
        ..Config::default()
    };
    let state = Arc::new(AppState { db: None, config });
    let app = create_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body();
    let bytes = body.collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(body["status"], "DEGRADED");
    assert_eq!(body["database_connected"], false);
    assert!(body["version"].is_string());
}
