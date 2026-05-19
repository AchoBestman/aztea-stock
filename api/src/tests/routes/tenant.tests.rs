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
