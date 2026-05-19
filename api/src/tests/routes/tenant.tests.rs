use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use std::sync::Arc;
use serde_json::{Value, json};
use sea_orm::{Statement, DatabaseBackend, ConnectionTrait};
use jsonwebtoken::{encode, Header, EncodingKey};

use crate::{create_app, AppState, config::Config, middleware::auth::Claims, tests::helpers::setup_test_db};

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
async fn test_set_tenant_two_factor_success_own_tenant() {
    let db = setup_test_db().await;
    
    // Seed
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'My Tenant', 'pharmacy', 'own@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Tenant admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-1', 'can_set_tenant_two_factor', 'Set 2FA', 'tenant')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-1')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User Admin', 'admin@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    let payload = json!({
        "tenant_id": "tenant-1",
        "two_factor_enabled": true
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/tenant/two-factor")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["id"], "tenant-1");
    assert_eq!(body["two_factor_enabled"], true);
}

#[tokio::test]
async fn test_set_tenant_two_factor_fails_unauthorized_different_tenant() {
    let db = setup_test_db().await;
    
    // Seed target tenant-2, caller tenant-1
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'My Tenant', 'pharmacy', 'own@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-2', 'Other Tenant', 'pharmacy', 'other@tenant.com', 0)".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Tenant admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-1', 'can_set_tenant_two_factor', 'Set 2FA', 'tenant')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-1')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User Admin', 'admin@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    let payload = json!({
        "tenant_id": "tenant-2",
        "two_factor_enabled": true
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/tenant/two-factor")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Must be Unauthorized/Forbidden because caller tenant-1 does not match target tenant-2 and caller is not from a system tenant
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_set_tenant_two_factor_success_system_tenant_modifying_another() {
    let db = setup_test_db().await;
    
    // Seed target tenant-2, caller is system tenant-1
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'System Tenant', 'pharmacy', 'system@tenant.com', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-2', 'Target Tenant', 'pharmacy', 'target@tenant.com', 0)".to_string())).await.unwrap();
    
    // Super Admin role automatically bypasses checking permission table or we can seed the permission
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'Super Admin', 'Super Admin')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'Super User', 'super@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-1", "tenant-1", "Super Admin", &config.jwt_secret);

    let payload = json!({
        "tenant_id": "tenant-2",
        "two_factor_enabled": true
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/tenant/two-factor")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Must be OK because caller is from the system tenant (is_system = true) and is Super Admin
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["id"], "tenant-2");
    assert_eq!(body["two_factor_enabled"], true);
}

#[tokio::test]
async fn test_set_tenant_two_factor_success_implicit_session_tenant() {
    let db = setup_test_db().await;
    
    // Seed
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'My Tenant', 'pharmacy', 'own@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Tenant admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-1', 'can_set_tenant_two_factor', 'Set 2FA', 'tenant')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-1')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User Admin', 'admin@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    // Payload completely omits tenant_id
    let payload = json!({
        "two_factor_enabled": true
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/tenant/two-factor")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["id"], "tenant-1");
    assert_eq!(body["two_factor_enabled"], true);
}

#[tokio::test]
async fn test_create_tenant_by_system_tenant_user_with_permission() {
    let db = setup_test_db().await;
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('system-tenant', 'System', 'pharmacy', 'sys@sys.com', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-sys', 'system-tenant', 'sysadmin', 'Sys Admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-create', 'can_create_tenant', 'Create Tenants', 'tenant')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-sys', 'perm-create')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-sys', 'system-tenant', 'Sys User', 'sys@example.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-sys', 'role-sys')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-sys", "system-tenant", "sysadmin", &config.jwt_secret);

    let payload = json!({
        "name": "New pharmacy",
        "business_type": "pharmacy",
        "email": "new@pharmacy.com",
        "country": "CG",
        "timezone": "Africa/Brazzaville"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/tenants")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["name"], "New pharmacy");
    assert_eq!(body["is_system"], false);
    assert_eq!(body["is_active"], true);
}

#[tokio::test]
async fn test_create_tenant_by_non_system_tenant_user_fails() {
    let db = setup_test_db().await;
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'Regular', 'pharmacy', 'reg@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-create', 'can_create_tenant', 'Create Tenants', 'tenant')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-create')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User', 'reg@example.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    let payload = json!({
        "name": "Forbidden Pharmacy",
        "business_type": "pharmacy",
        "email": "forbidden@pharmacy.com"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/tenants")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_update_tenant_own_fields_success_system_fields_fail() {
    let db = setup_test_db().await;
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'Regular', 'pharmacy', 'reg@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-update', 'can_update_tenant', 'Update Tenant', 'tenant')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-update')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User', 'reg@example.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    // 1. Updating own non-system fields should succeed
    let payload_own = json!({
        "phone": "123456789",
        "timezone": "Africa/Brazzaville",
        "two_factor_enabled": true
    });

    let response_own = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/admin/tenant")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload_own).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response_own.status(), StatusCode::OK);
    let bytes = response_own.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["phone"], "123456789");
    assert_eq!(body["timezone"], "Africa/Brazzaville");
    assert_eq!(body["two_factor_enabled"], true);

    // 2. Attempting to update system fields like name should fail
    let payload_system = json!({
        "name": "Malicious Change"
    });

    let response_system = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/admin/tenant")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload_system).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response_system.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_update_tenant_credentials_permission() {
    let db = setup_test_db().await;
    
    // System tenant and user
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('system-tenant', 'System', 'pharmacy', 'sys@sys.com', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('target-tenant', 'Target', 'pharmacy', 'target@sys.com', 0)".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-sys', 'system-tenant', 'sysadmin', 'Sys Admin')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-update', 'can_update_tenant', 'Update Tenant', 'tenant')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-creds', 'can_update_tenant_credentials', 'Update SMTP Creds', 'tenant')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-sys', 'perm-update')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-sys', 'system-tenant', 'Sys User', 'sys@example.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-sys', 'role-sys')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db.clone()), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-sys", "system-tenant", "sysadmin", &config.jwt_secret);

    // 1. Without can_update_tenant_credentials permission, attempting to update sender_email fails
    let payload = json!({
        "sender_email": "sender@smtp.com"
    });

    let response1 = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/admin/tenant?tenant_id=target-tenant")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::UNAUTHORIZED);

    // 2. Grant can_update_tenant_credentials to role-sys
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-sys', 'perm-creds')".to_string())).await.unwrap();

    let response2 = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/admin/tenant?tenant_id=target-tenant")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::OK);
    let bytes = response2.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["sender_email"], "sender@smtp.com");
}

#[tokio::test]
async fn test_soft_delete_and_login_prevention() {
    let db = setup_test_db().await;
    
    // Seed system tenant and target tenant
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('system-tenant', 'System', 'pharmacy', 'sys@sys.com', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system, is_active) VALUES ('target-tenant', 'Target', 'pharmacy', 'target@sys.com', 0, 1)".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-sys', 'system-tenant', 'sysadmin', 'Sys Admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-target', 'target-tenant', 'targetadmin', 'Target Admin')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-delete', 'can_delete_tenant', 'Delete Tenants', 'tenant')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-read', 'can_read_tenant', 'Read Tenants', 'tenant')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-sys', 'perm-delete')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-target', 'perm-read')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-sys', 'system-tenant', 'Sys User', 'sys@example.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-sys', 'role-sys')".to_string())).await.unwrap();
    
    // Hash password "password" for target-user
    let password_hash = bcrypt::hash("password", bcrypt::DEFAULT_COST).unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, format!("INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-target', 'target-tenant', 'Target User', 'target@example.com', '{}', 1)", password_hash))).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-target', 'role-target')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db.clone()), config: config.clone() });
    let app = create_app(state);

    // 1. Initial login for target-user should work
    let login_payload = json!({
        "email": "target@example.com",
        "password": "password"
    });

    let login_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    // 2. Soft-delete target-tenant using sys-user token
    let sys_token = create_token("user-sys", "system-tenant", "sysadmin", &config.jwt_secret);

    let delete_response = app.clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/admin/tenants/target-tenant")
                .header("Authorization", format!("Bearer {}", sys_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);

    // 3. Login for target-user must now fail because tenant is inactive
    let login_response2 = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login_response2.status(), StatusCode::UNAUTHORIZED);

    // 4. Any further action from logged-in target-user must also be rejected by extract_tenant middleware
    let target_token = create_token("user-target", "target-tenant", "targetadmin", &config.jwt_secret);

    let action_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/tenant")
                .header("Authorization", format!("Bearer {}", target_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(action_response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_tenants_advanced_filters() {
    let db = setup_test_db().await;

    // Seed system tenant
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system, is_active, created_at) VALUES ('system-tenant', 'System', 'both', 'sys@sys.com', 1, 1, '2026-05-19T12:00:00+00:00')".to_string())).await.unwrap();
    
    // Seed tenant 1
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, phone, country, address, is_system, is_active, created_at) VALUES ('tenant-1', 'Super Pharmacy Store', 'pharmacy', 'pharmacy@sys.com', '123456', 'CG', 'Brazzaville street', 0, 1, '2026-05-19T10:00:00+00:00')".to_string())).await.unwrap();

    // Seed tenant 2
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, phone, country, address, is_system, is_active, created_at) VALUES ('tenant-2', 'Alpha Supermarket', 'supermarket', 'super@sys.com', '789101', 'FR', 'Paris boulevard', 0, 0, '2026-05-19T14:00:00+00:00')".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-sys', 'system-tenant', 'sysadmin', 'Sys Admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-read', 'can_read_tenant', 'Read Tenants', 'tenant')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-sys', 'perm-read')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-sys', 'system-tenant', 'Sys User', 'sys@example.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-sys', 'role-sys')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db.clone()), config: config.clone() });
    let app = create_app(state);
    let sys_token = create_token("user-sys", "system-tenant", "sysadmin", &config.jwt_secret);

    // Helper to request list_tenants with custom query string
    let get_filtered = |query_str: &str| {
        let app_clone = app.clone();
        let token = sys_token.clone();
        let uri = format!("/api/v1/admin/tenants{}", query_str);
        async move {
            let res = app_clone
                .oneshot(
                    Request::builder()
                        .method("GET")
                        .uri(uri)
                        .header("Authorization", format!("Bearer {}", token))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            
            assert_eq!(res.status(), StatusCode::OK);
            let bytes = res.into_body().collect().await.unwrap().to_bytes();
            let parsed: Value = serde_json::from_slice(&bytes).unwrap();
            parsed.as_array().unwrap().clone()
        }
    };

    // 1. Unfiltered: should return all 3 tenants
    let all = get_filtered("").await;
    assert_eq!(all.len(), 3);

    // 2. Filter by business_type = pharmacy
    let pharmacies = get_filtered("?business_type=pharmacy").await;
    assert_eq!(pharmacies.len(), 1);
    assert_eq!(pharmacies[0]["id"], "tenant-1");

    // 3. Search query: match name (Super Pharmacy Store)
    let search_name = get_filtered("?search=Pharmacy").await;
    assert_eq!(search_name.len(), 1);
    assert_eq!(search_name[0]["id"], "tenant-1");

    // 4. Search query: match email or name containing 'super' (matches tenant-1 and tenant-2)
    let search_super = get_filtered("?search=super").await;
    assert_eq!(search_super.len(), 2);

    // 5. Search query: match name containing 'supermarket' (matches only tenant-2)
    let search_sm = get_filtered("?search=supermarket").await;
    assert_eq!(search_sm.len(), 1);
    assert_eq!(search_sm[0]["id"], "tenant-2");

    // 6. Search query: match address (Brazzaville street)
    let search_addr = get_filtered("?search=Brazzaville").await;
    assert_eq!(search_addr.len(), 1);
    assert_eq!(search_addr[0]["id"], "tenant-1");

    // 6. Filter by status: is_active = false
    let inactive = get_filtered("?is_active=false").await;
    assert_eq!(inactive.len(), 1);
    assert_eq!(inactive[0]["id"], "tenant-2");

    // 7. Filter by date interval: created_after
    let created_after = get_filtered("?created_after=2026-05-19T11:00:00Z").await;
    // system-tenant (12:00) and tenant-2 (14:00) match
    assert_eq!(created_after.len(), 2);
    let ids: Vec<&str> = created_after.iter().map(|t| t["id"].as_str().unwrap()).collect();
    assert!(ids.contains(&"system-tenant"));
    assert!(ids.contains(&"tenant-2"));

    // 8. Filter by date interval: created_before
    let created_before = get_filtered("?created_before=2026-05-19T11:00:00Z").await;
    // only tenant-1 (10:00) matches
    assert_eq!(created_before.len(), 1);
    assert_eq!(created_before[0]["id"], "tenant-1");

    // 9. Flexible Boolean parsing tests
    // '0' should match 'false' (tenant-2)
    let inactive_digit = get_filtered("?is_active=0").await;
    assert_eq!(inactive_digit.len(), 1);
    assert_eq!(inactive_digit[0]["id"], "tenant-2");

    // '1' should match 'true' (tenant-1 and system-tenant)
    let active_digit = get_filtered("?is_active=1").await;
    assert_eq!(active_digit.len(), 2);

    // 10. Date-only ISO parsing (YYYY-MM-DD)
    let created_after_iso = get_filtered("?created_after=2026-05-19").await;
    assert_eq!(created_after_iso.len(), 3); // all three created on 2026-05-19

    // 11. Error handling: invalid is_active format should throw 400 Bad Request
    let res_err = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/tenants?is_active=xyz")
                .header("Authorization", format!("Bearer {}", sys_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_err.status(), StatusCode::BAD_REQUEST);

    // 12. Error handling: invalid date format should throw 400 Bad Request
    let res_date_err = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/tenants?created_after=invalid-date")
                .header("Authorization", format!("Bearer {}", sys_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_date_err.status(), StatusCode::BAD_REQUEST);
}
