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

#[tokio::test]
async fn test_list_roles_with_tenant_filter_system_tenant() {
    let db = setup_test_db().await;
    
    // Seed system tenant and a regular tenant
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('system-tenant', 'System', 'pharmacy', 'system@system.com', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-2', 'Other Tenant', 'pharmacy', 'other@tenant.com', 0)".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-sys', 'system-tenant', 'sysadmin', 'System Admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-other', 'tenant-2', 'clerk', 'Clerk')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-read', 'can_read_role', 'Read roles', 'roles')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-sys', 'perm-read')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-sys', 'system-tenant', 'Sys User', 'sys@example.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-sys', 'role-sys')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-sys", "system-tenant", "sysadmin", &config.jwt_secret);

    // List roles with no filter (system tenant sees all roles)
    let response_all = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/roles")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response_all.status(), StatusCode::OK);
    let bytes_all = response_all.into_body().collect().await.unwrap().to_bytes();
    let body_all: Value = serde_json::from_slice(&bytes_all).unwrap();
    let arr_all = body_all.as_array().unwrap();
    assert_eq!(arr_all.len(), 2);

    // List roles filtered by tenant-2
    let response_filtered = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/roles?tenant_id=tenant-2")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response_filtered.status(), StatusCode::OK);
    let bytes_filtered = response_filtered.into_body().collect().await.unwrap().to_bytes();
    let body_filtered: Value = serde_json::from_slice(&bytes_filtered).unwrap();
    let arr_filtered = body_filtered.as_array().unwrap();
    assert_eq!(arr_filtered.len(), 1);
    assert_eq!(arr_filtered[0]["id"], "role-other");
    assert_eq!(arr_filtered[0]["name"], "clerk");
}

#[tokio::test]
async fn test_list_roles_with_name_search() {
    let db = setup_test_db().await;
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'My Tenant', 'pharmacy', 'own@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Tenant admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-2', 'tenant-1', 'cashier', 'Tenant cashier')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-read', 'can_read_role', 'Read roles', 'roles')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-read')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User Admin', 'admin@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    // Search by name "cash"
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/roles?name=cash")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], "role-2");
    assert_eq!(arr[0]["name"], "cashier");
}

#[tokio::test]
async fn test_non_system_tenant_forbidden_from_other_tenant_role() {
    let db = setup_test_db().await;
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'Tenant One', 'pharmacy', 't1@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-2', 'Tenant Two', 'pharmacy', 't2@tenant.com', 0)".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Tenant one admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-2', 'tenant-2', 'manager', 'Tenant two manager')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-all', 'can_read_role', 'Read roles', 'roles')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-all')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User One', 'user1@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    // Try accessing role-2 (which belongs to tenant-2)
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/roles/role-2")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Non-system user must be blocked with Unauthorized (401)
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_super_admin_role_restrictions() {
    let db = setup_test_db().await;
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'My Tenant', 'pharmacy', 'own@tenant.com', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Tenant admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('super-admin-role', 'tenant-1', 'Super Admin', 'Supreme Admin')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('p-create', 'can_create_role', 'Create', 'roles')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('p-update', 'can_update_role', 'Update', 'roles')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('p-delete', 'can_delete_role', 'Delete', 'roles')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'p-create')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'p-update')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'p-delete')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User Admin', 'admin@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);
    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    // 1. Refuse creating a role with name 'Super Admin'
    let create_payload = json!({
        "name": "Super Admin",
        "description": "API-created Super Admin role"
    });

    let res_create = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/roles")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_create.status(), StatusCode::BAD_REQUEST);

    // 2. Refuse renaming an existing role to 'Super Admin'
    let update_payload = json!({
        "name": "Super Admin",
        "description": "Rename to super admin"
    });

    let res_rename = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/admin/roles/role-1")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&update_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_rename.status(), StatusCode::BAD_REQUEST);

    // 3. Refuse modifying the seeded 'Super Admin' role
    let res_modify_sa = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/admin/roles/super-admin-role")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&json!({"name": "Another Name"})).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_modify_sa.status(), StatusCode::BAD_REQUEST);

    // 4. Refuse deleting the seeded 'Super Admin' role
    let res_delete_sa = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/admin/roles/super-admin-role")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_delete_sa.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_assign_role_permissions_success_and_errors() {
    let db = setup_test_db().await;
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'My Tenant', 'pharmacy', 'own@tenant.com', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Tenant admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-target', 'tenant-1', 'cashier', 'Tenant cashier')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('super-admin-role', 'tenant-1', 'Super Admin', 'Supreme')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('p-read', 'can_read_role', 'Read', 'roles')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('p-update', 'can_update_role', 'Update', 'roles')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-sale-1', 'can_create_sale', 'Create Sales', 'sales')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-sale-2', 'can_read_sale', 'Read Sales', 'sales')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'p-read')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'p-update')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User Admin', 'admin@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);
    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    // 1. Success: sync permissions to role-target
    let assign_payload = json!({
        "permission_ids": vec!["perm-sale-1", "perm-sale-2"]
    });

    let res_assign = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/roles/role-target/permissions")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&assign_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_assign.status(), StatusCode::OK);

    // 2. Refuse sync if one or more permissions are invalid/non-existent
    let bad_assign_payload = json!({
        "permission_ids": vec!["perm-sale-1", "invalid-permission-id"]
    });

    let res_bad_assign = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/roles/role-target/permissions")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&bad_assign_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_bad_assign.status(), StatusCode::BAD_REQUEST);

    // 3. Refuse sync for the seeded 'Super Admin' role
    let res_sa_assign = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/roles/super-admin-role/permissions")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&assign_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_sa_assign.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_grouped_permissions_success_and_forbidden() {
    let db = setup_test_db().await;
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'My Tenant', 'pharmacy', 'own@tenant.com', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-authorized', 'tenant-1', 'admin', 'Tenant admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-unauthorized', 'tenant-1', 'cashier', 'Tenant cashier')".to_string())).await.unwrap();
    
    // Seed some permissions in different groups
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('p-read', 'can_read_role', 'Read Roles', 'roles')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('p-read-perm', 'can_read_permission', 'Read Perms', 'roles')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-sale-1', 'can_create_sale', 'Create Sales', 'sales')".to_string())).await.unwrap();
    
    // Assign permissions
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-authorized', 'p-read-perm')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-unauthorized', 'p-read')".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-auth', 'tenant-1', 'Authorized User', 'auth@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-unauth', 'tenant-1', 'Unauthorized User', 'unauth@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-auth', 'role-authorized')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-unauth', 'role-unauthorized')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);
    
    let token_auth = create_token("user-auth", "tenant-1", "admin", &config.jwt_secret);
    let token_unauth = create_token("user-unauth", "tenant-1", "cashier", &config.jwt_secret);

    // 1. Unauthorized: should return 401 (ApiError::Unauthorized maps to 401 in Axum)
    let res_unauth = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/permissions")
                .header("Authorization", format!("Bearer {}", token_unauth))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_unauth.status(), StatusCode::UNAUTHORIZED);

    // 2. Authorized: should successfully retrieve grouped permissions
    let res_auth = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/permissions")
                .header("Authorization", format!("Bearer {}", token_auth))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_auth.status(), StatusCode::OK);
    
    let bytes = res_auth.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let arr = body.as_array().unwrap();
    
    // We seeded permissions in groups "roles" and "sales", sorted by group name ("roles" < "sales")
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["group"], "roles");
    assert_eq!(arr[0]["permissions"].as_array().unwrap().len(), 2);
    
    assert_eq!(arr[1]["group"], "sales");
    assert_eq!(arr[1]["permissions"].as_array().unwrap().len(), 1);
}
