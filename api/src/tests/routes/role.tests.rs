use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use std::sync::Arc;
use serde_json::Value;
use sea_orm::{Database, ConnectionTrait, Statement, DatabaseBackend, DatabaseConnection};
use jsonwebtoken::{encode, Header, EncodingKey};

use crate::{create_app, AppState, config::Config, middleware::auth::Claims};

async fn setup_test_db() -> DatabaseConnection {
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

    db
}

fn create_token(user_id: &str, tenant_id: &str, role: &str, jwt_secret: &str) -> String {
    let claims = Claims {
        sub: user_id.to_string(),
        tenant_id: tenant_id.to_string(),
        role: role.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_bytes())).unwrap()
}

#[tokio::test]
async fn test_delete_role_fails_when_assigned_to_user() {
    let db = setup_test_db().await;
    
    // Seed
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'My Tenant', 'pharmacy', 'own@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Tenant admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-2', 'tenant-1', 'user_role', 'Standard User')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-1', 'can_delete_role', 'Delete roles', 'roles')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-1')".to_string())).await.unwrap();
    
    // Assign role-1 to user-1, role-2 to user-2
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User Admin', 'admin@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-2', 'tenant-1', 'User Two', 'user2@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-2', 'role-2')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    // Try deleting role-2 which is assigned to user-2
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/admin/roles/role-2")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Must be Bad Request because role-2 is assigned to user-2
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Ce rôle ne peut pas être supprimé car il est actuellement attribué à un ou plusieurs utilisateurs.")
    );
}

#[tokio::test]
async fn test_delete_role_success_when_not_assigned() {
    let db = setup_test_db().await;
    
    // Seed
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'My Tenant', 'pharmacy', 'own@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Tenant admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-2', 'tenant-1', 'unassigned_role', 'Unassigned Role')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-1', 'can_delete_role', 'Delete roles', 'roles')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-1')".to_string())).await.unwrap();
    
    // Only assign role-1 to user-1. role-2 has no users.
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User Admin', 'admin@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    // Delete role-2
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/admin/roles/role-2")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["success"], true);
}
