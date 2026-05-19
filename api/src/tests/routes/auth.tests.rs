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

fn create_token(user_id: &str, tenant_id: &str, role: &str, secret: &str) -> String {
    let claims = crate::middleware::auth::Claims {
        sub: user_id.to_string(),
        tenant_id: tenant_id.to_string(),
        role: role.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

#[tokio::test]
async fn test_profile_get_and_update() {
    let db = crate::tests::helpers::setup_test_db().await;

    // Seed test data
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'Pharmacie Test', 'both', 'test@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-2', 'Tenant 2', 'both', 't2@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-sys', 'System Tenant', 'both', 'sys@tenant.com', 1)".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'manager', 'Test role manager')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-sys-sa', 'tenant-sys', 'Super Admin', 'Super Admin System')".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('p-status', 'can_update_user_status', 'Update User Status', 'users')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('p-2fa', 'can_update_user_two_factor', 'Update User 2FA', 'users')".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'p-status')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'p-2fa')".to_string())).await.unwrap();

    // user-1 has role-1 (manager, which has the required update permissions in tenant-1)
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active, two_factor_enabled) VALUES ('user-1', 'tenant-1', 'Jean Moukala', 'test@example.com', 'hash', 1, 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    // user-2 has no permissions in tenant-1
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active, two_factor_enabled) VALUES ('user-2', 'tenant-1', 'Paul Dupont', 'paul@example.com', 'hash', 1, 0)".to_string())).await.unwrap();

    // user-other belongs to tenant-2 (different tenant)
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active, two_factor_enabled) VALUES ('user-other', 'tenant-2', 'Foreigner', 'foreign@example.com', 'hash', 1, 0)".to_string())).await.unwrap();

    // user-sys-sa is the system super admin
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active, two_factor_enabled) VALUES ('user-sys-sa', 'tenant-sys', 'Sys Admin', 'admin@example.com', 'hash', 1, 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-sys-sa', 'role-sys-sa')".to_string())).await.unwrap();

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
    
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token_u1 = create_token("user-1", "tenant-1", "manager", &config.jwt_secret);
    let token_u2 = create_token("user-2", "tenant-1", "employee", &config.jwt_secret);
    let token_sys = create_token("user-sys-sa", "tenant-sys", "Super Admin", &config.jwt_secret);

    // 1. GET /api/v1/auth/profile - Unauthorized
    let res_get_unauth = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/auth/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_get_unauth.status(), StatusCode::UNAUTHORIZED);

    // 2. GET /api/v1/auth/profile - Success (user-1)
    let res_get_success = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/auth/profile")
                .header("Authorization", format!("Bearer {}", token_u1))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_get_success.status(), StatusCode::OK);
    let bytes = res_get_success.into_body().collect().await.unwrap().to_bytes();
    let profile: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(profile["id"], "user-1");
    assert_eq!(profile["name"], "Jean Moukala");
    assert_eq!(profile["tenant"]["name"], "Pharmacie Test");
    assert_eq!(profile["tenant"]["business_type"], "both");

    // 3. PUT /api/v1/auth/profile - Update own name (Success)
    let payload_name = json!({
        "name": "Jean Moukala Updated"
    });
    let res_put_name = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/auth/profile")
                .header("Authorization", format!("Bearer {}", token_u1))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload_name).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_put_name.status(), StatusCode::OK);
    let bytes_name = res_put_name.into_body().collect().await.unwrap().to_bytes();
    let profile_name: Value = serde_json::from_slice(&bytes_name).unwrap();
    assert_eq!(profile_name["name"], "Jean Moukala Updated");

    // 4. PUT /api/v1/auth/profile - Try to update own status/2FA without permission (user-2)
    let payload_status = json!({
        "is_active": false
    });
    let res_put_forbidden = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/auth/profile")
                .header("Authorization", format!("Bearer {}", token_u2))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload_status).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_put_forbidden.status(), StatusCode::FORBIDDEN);

    // 5. PUT /api/v1/auth/profile - Update another user's status/2FA in same tenant with permission (user-1 updating user-2)
    let payload_u2_update = json!({
        "user_id": "user-2",
        "is_active": false,
        "two_factor_enabled": true
    });
    let res_put_other_ok = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/auth/profile")
                .header("Authorization", format!("Bearer {}", token_u1))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload_u2_update).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_put_other_ok.status(), StatusCode::OK);
    let bytes_u2 = res_put_other_ok.into_body().collect().await.unwrap().to_bytes();
    let profile_u2: Value = serde_json::from_slice(&bytes_u2).unwrap();
    assert_eq!(profile_u2["id"], "user-2");
    assert_eq!(profile_u2["is_active"], false);
    assert_eq!(profile_u2["two_factor_enabled"], true);

    // 6. PUT /api/v1/auth/profile - Try to update another user's name (user-1 trying to change user-2's name) -> Forbidden
    let payload_u2_name = json!({
        "user_id": "user-2",
        "name": "Malicious Change"
    });
    let res_put_u2_name = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/auth/profile")
                .header("Authorization", format!("Bearer {}", token_u1))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload_u2_name).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_put_u2_name.status(), StatusCode::FORBIDDEN);

    // 7. PUT /api/v1/auth/profile - Try to update user in another tenant -> Unauthorized
    let payload_other_tenant = json!({
        "user_id": "user-other",
        "name": "Attempt"
    });
    let res_put_other_tenant = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/auth/profile")
                .header("Authorization", format!("Bearer {}", token_u1))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload_other_tenant).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_put_other_tenant.status(), StatusCode::UNAUTHORIZED);

    // 8. PUT /api/v1/auth/profile - System Super Admin can do anything to anyone
    let payload_sys_any = json!({
        "user_id": "user-other",
        "name": "Renamed By System Admin",
        "is_active": false,
        "two_factor_enabled": true
    });
    let res_sys_any = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/auth/profile")
                .header("Authorization", format!("Bearer {}", token_sys))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload_sys_any).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_sys_any.status(), StatusCode::OK);
    let bytes_sys = res_sys_any.into_body().collect().await.unwrap().to_bytes();
    let profile_sys: Value = serde_json::from_slice(&bytes_sys).unwrap();
    assert_eq!(profile_sys["id"], "user-other");
    assert_eq!(profile_sys["name"], "Renamed By System Admin");
    assert_eq!(profile_sys["is_active"], false);
    assert_eq!(profile_sys["two_factor_enabled"], true);
}
