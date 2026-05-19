use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use std::sync::Arc;
use serde_json::{Value, json};
use sea_orm::{ConnectionTrait, Statement, DatabaseBackend};
use bcrypt::{hash, DEFAULT_COST};

use crate::{create_app, AppState, config::Config};

#[tokio::test]
async fn test_login_success() {
    // 1. Setup in-memory database
    let db = crate::tests::helpers::setup_test_db().await;

    // Seed test data
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'Pharmacie Test', 'both', 'test@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'manager', 'Test role manager')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-1', 'can_create_product', 'Test permission', 'products')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-1')".to_string())).await.unwrap();
    
    let hashed_pw = hash("password123", DEFAULT_COST).unwrap();
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        format!("INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'Jean Moukala', 'test@example.com', '{}', 1)", hashed_pw)
    )).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

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
    
    let state = Arc::new(AppState { db: Some(db), config });
    let app = create_app(state);

    let payload = json!({
        "email": "test@example.com",
        "password": "password123"
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

    assert!(body["access_token"].as_str().is_some());
    assert!(body["refresh_token"].as_str().is_some());
    assert_eq!(body["expires_in"], 3600);
    assert_eq!(body["user"]["id"], "user-1");
    assert_eq!(body["user"]["name"], "Jean Moukala");
    assert_eq!(body["user"]["role"], "manager");
    assert_eq!(body["user"]["tenant_id"], "tenant-1");
    assert_eq!(body["user"]["tenant_name"], "Pharmacie Test");
    
    let roles_arr = body["user"]["roles"].as_array().unwrap();
    assert_eq!(roles_arr.len(), 1);
    assert_eq!(roles_arr[0], "manager");

    let perms_arr = body["user"]["permissions"].as_array().unwrap();
    assert_eq!(perms_arr.len(), 1);
    assert_eq!(perms_arr[0], "can_create_product");
}
