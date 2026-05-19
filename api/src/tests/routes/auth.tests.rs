use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use std::sync::Arc;
use serde_json::{Value, json};

use crate::{create_app, AppState, config::Config};

#[tokio::test]
async fn test_login_success() {
    let config = Config {
        database_url: None,
        jwt_secret: "test_jwt_secret_123456_test_jwt_secret".to_string(),
        port: 8080,
        rust_log: "info".to_string(),
    };
    let state = Arc::new(AppState { db: None, config });
    let app = create_app(state);

    let payload = json!({
        "email": "test@example.com",
        "password": "password123",
        "license_key": "AZTEASTOCK-12345-67890-ABCDE"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body();
    let bytes = body.collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(body["access_token"], "mock_jwt_token");
    assert_eq!(body["refresh_token"], "mock_refresh_token");
    assert_eq!(body["expires_in"], 3600);
    assert_eq!(body["user"]["role"], "manager");
}
