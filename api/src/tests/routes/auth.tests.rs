use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use std::sync::Arc;
use serde_json::{Value, json};
use sea_orm::{Database, ConnectionTrait, Statement, DatabaseBackend};
use bcrypt::{hash, DEFAULT_COST};

use crate::{create_app, AppState, config::Config};

#[tokio::test]
async fn test_login_success() {
    // 1. Setup in-memory database
    let db = Database::connect("sqlite::memory:").await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE tenants (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            business_type VARCHAR(50) NOT NULL CHECK (business_type IN ('pharmacy','supermarket','both')),
            email VARCHAR(255) UNIQUE NOT NULL,
            phone VARCHAR(50),
            address TEXT,
            country VARCHAR(100) DEFAULT 'CG',
            timezone VARCHAR(100) DEFAULT 'Africa/Brazzaville',
            logo_url TEXT,
            is_active BOOLEAN DEFAULT true,
            is_system BOOLEAN DEFAULT false NOT NULL,
            two_factor_enabled BOOLEAN DEFAULT false NOT NULL,
            sender_email TEXT,
            sender_user TEXT,
            sender_password TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE roles (
            id VARCHAR(36) PRIMARY KEY,
            tenant_id VARCHAR(36) NOT NULL,
            name VARCHAR(50) NOT NULL,
            description TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
            CONSTRAINT uniq_tenant_role_name UNIQUE (tenant_id, name)
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE users (
            id VARCHAR(36) PRIMARY KEY,
            tenant_id VARCHAR(36) NOT NULL,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            pin_hash VARCHAR(255),
            is_active BOOLEAN DEFAULT true,
            last_login TIMESTAMPTZ,
            two_factor_enabled BOOLEAN DEFAULT false NOT NULL,
            two_factor_code VARCHAR(10),
            two_factor_expires_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
            UNIQUE (tenant_id, email)
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE user_roles (
            user_id VARCHAR(36) NOT NULL,
            role_id VARCHAR(36) NOT NULL,
            PRIMARY KEY (user_id, role_id),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE permissions (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(100) UNIQUE NOT NULL,
            description TEXT,
            model_group VARCHAR(50) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE role_permissions (
            role_id VARCHAR(36) NOT NULL,
            permission_id VARCHAR(36) NOT NULL,
            PRIMARY KEY (role_id, permission_id),
            FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
            FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
        );
    ".to_string())).await.unwrap();

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
