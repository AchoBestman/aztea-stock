use axum::{
    http::StatusCode,
    response::IntoResponse,
};
use http_body_util::BodyExt;
use serde_json::Value;

use crate::errors::ApiError;

async fn parse_response_body(response: axum::response::Response) -> Value {
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body_bytes).unwrap()
}

#[tokio::test]
async fn test_not_found_error() {
    let err = ApiError::NotFound("product".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = parse_response_body(response).await;
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "NOT_FOUND");
    assert_eq!(body["error"]["message"], "Not found: product");
}

#[tokio::test]
async fn test_unauthorized_error() {
    let err = ApiError::Unauthorized("invalid token".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = parse_response_body(response).await;
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
    assert_eq!(body["error"]["message"], "Unauthorized: invalid token");
}

#[tokio::test]
async fn test_bad_request_error() {
    let err = ApiError::BadRequest("missing field".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = parse_response_body(response).await;
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "Bad request: missing field");
}

#[tokio::test]
async fn test_internal_error() {
    let err = ApiError::Internal("disk full".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = parse_response_body(response).await;
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "INTERNAL_SERVER_ERROR");
    assert_eq!(body["error"]["message"], "Internal server error: disk full");
}

#[tokio::test]
async fn test_database_error() {
    // Generate a dummy SQLx error (RowNotFound is a simple variant)
    let sqlx_err = sqlx::Error::RowNotFound;
    let err = ApiError::Database(sqlx_err);
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = parse_response_body(response).await;
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "DATABASE_ERROR");
    assert_eq!(body["error"]["message"], "Database error: no rows returned by a query that expected to return at least one row");
}
