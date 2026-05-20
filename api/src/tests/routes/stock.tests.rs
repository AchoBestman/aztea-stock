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
async fn test_stock_lifecycle_and_calculations() {
    let db = setup_test_db().await;

    // Seed tenant, role, permissions, user
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'My Tenant', 'pharmacy', 'own@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Tenant admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-read', 'can_read_stock', 'Read stock', 'stock')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-write', 'can_manage_stock', 'Manage stock', 'stock')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-read')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-write')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User Admin', 'admin@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    // Seed products
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        INSERT INTO products (id, tenant_id, name, unit, purchase_price, selling_price, is_active, requires_prescription, created_at, updated_at)
        VALUES ('product-1', 'tenant-1', 'Smartphone Pro', 'unité', 500.0, 799.0, 1, 0, '2026-05-20T10:00:00Z', '2026-05-20T10:00:00Z')
    ".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    // 1. Create Stock Item
    let payload_item = json!({
        "product_id": "product-1",
        "quantity": 10.0,
        "low_stock_threshold": 3.0,
        "unit_location": "Aisle 4, Shelf B",
        "batch_number": "BATCH-100",
        "expiry_date": "2027-05-20"
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/stock/items")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload_item).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let item: Value = serde_json::from_slice(&bytes).unwrap();
    let item_id = item["id"].as_str().unwrap();
    assert_eq!(item["product_id"], "product-1");
    assert_eq!(item["quantity"], 10.0);
    assert_eq!(item["unit_location"], "Aisle 4, Shelf B");

    // 2. Read Stock Item Detail
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/stock/items/{}", item_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let fetched: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(fetched["id"], item_id);
    assert_eq!(fetched["product_name"], "Smartphone Pro");

    // 3. Update Stock Item (manual edit of low_stock_threshold)
    let payload_update = json!({
        "low_stock_threshold": 5.0
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/stock/items/{}", item_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload_update).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let updated: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(updated["low_stock_threshold"], 5.0);

    // 4. Create Stock Movement (increase quantity by 5 via purchase)
    let payload_movement = json!({
        "product_id": "product-1",
        "movement_type": "purchase",
        "quantity_change": 5.0,
        "note": "Re-stocking"
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/stock/movements")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload_movement).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let movement: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(movement["movement_type"], "purchase");
    assert_eq!(movement["quantity_before"], 10.0);
    assert_eq!(movement["quantity_change"], 5.0);
    assert_eq!(movement["quantity_after"], 15.0);

    // 5. Verify Stock Item Quantity has updated to 15.0
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/stock/items/{}", item_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let fetched_after: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(fetched_after["quantity"], 15.0);

    // 6. List Stock Movements
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/stock/movements?product_id=product-1")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let list_res: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(list_res["total"].as_u64().unwrap() >= 2); // Initial (created since quantity > 0) + Purchase

    // 7. Delete Stock Item
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/stock/items/{}", item_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
