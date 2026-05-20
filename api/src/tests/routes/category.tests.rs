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
async fn test_category_lifecycle_and_filtering() {
    let db = setup_test_db().await;

    // Seed
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'My Tenant', 'pharmacy', 'own@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Tenant admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-read', 'can_read_category', 'Read categories', 'category')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-create', 'can_create_category', 'Create categories', 'category')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-update', 'can_update_category', 'Update categories', 'category')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-read')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-create')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-update')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User Admin', 'admin@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    // 1. Create Parent Category
    let payload_parent = json!({
        "name": "Parent Cat",
        "description": "I am parent"
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/categories")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload_parent).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let parent_cat: Value = serde_json::from_slice(&bytes).unwrap();
    let parent_id = parent_cat["id"].as_str().unwrap();
    assert_eq!(parent_cat["name"], "Parent Cat");
    assert!(parent_cat["parent_name"].is_null());

    // 2. Create Child Category
    let payload_child = json!({
        "name": "Child Cat",
        "description": "I am child",
        "parent_id": parent_id
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/categories")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload_child).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let child_cat: Value = serde_json::from_slice(&bytes).unwrap();
    let child_id = child_cat["id"].as_str().unwrap();
    assert_eq!(child_cat["name"], "Child Cat");
    assert_eq!(child_cat["parent_name"], "Parent Cat");

    // 3. Edit (Update) Category and verify parent_name is in response
    let payload_edit = json!({
        "name": "Updated Child Cat"
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/categories/{}", child_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload_edit).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let updated_cat: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(updated_cat["name"], "Updated Child Cat");
    assert_eq!(updated_cat["parent_name"], "Parent Cat");

    // 4. List all categories (paginated)
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/categories")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let paginated_result: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(paginated_result["total"], 2);
    let list = paginated_result["data"].as_array().unwrap();
    assert_eq!(list.len(), 2);

    // Verify parent_name is populated in the list
    let parent_item = list.iter().find(|c| c["id"].as_str().unwrap() == parent_id).unwrap();
    assert!(parent_item["parent_name"].is_null());
    let child_item = list.iter().find(|c| c["id"].as_str().unwrap() == child_id).unwrap();
    assert_eq!(child_item["parent_name"], "Parent Cat");

    // 5. Filter categories by parent_id = parent_id
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/categories?parent_id={}", parent_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let paginated_result: Value = serde_json::from_slice(&bytes).unwrap();
    let filtered_list = paginated_result["data"].as_array().unwrap();
    assert_eq!(filtered_list.len(), 1);
    assert_eq!(filtered_list[0]["id"].as_str().unwrap(), child_id);
    assert_eq!(filtered_list[0]["parent_name"], "Parent Cat");

    // 6. Filter categories by parent_id = null
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/categories?parent_id=null")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let paginated_result: Value = serde_json::from_slice(&bytes).unwrap();
    let root_list = paginated_result["data"].as_array().unwrap();
    assert_eq!(root_list.len(), 1);
    assert_eq!(root_list[0]["id"].as_str().unwrap(), parent_id);
    assert!(root_list[0]["parent_name"].is_null());

    // 7. Search category by name
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/categories?search=Child")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let paginated_result: Value = serde_json::from_slice(&bytes).unwrap();
    let search_list = paginated_result["data"].as_array().unwrap();
    assert_eq!(search_list.len(), 1);
    assert_eq!(search_list[0]["id"].as_str().unwrap(), child_id);

    // 8. Filter category by is_active
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/categories?is_active=false")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let paginated_result: Value = serde_json::from_slice(&bytes).unwrap();
    let inactive_list = paginated_result["data"].as_array().unwrap();
    assert_eq!(inactive_list.len(), 0);
}
