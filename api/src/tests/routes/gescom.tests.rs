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
async fn test_gescom_sales_and_alerts_lifecycle() {
    let db = setup_test_db().await;

    // Seed tenant, role, permissions, user
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'My Tenant', 'pharmacy', 'own@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Tenant admin')".to_string())).await.unwrap();
    
    let permissions = vec![
        "can_create_sale", "can_read_sale", "can_update_sale", "can_delete_sale",
        "can_read_alert", "can_manage_alert", "can_read_stock", "can_manage_stock"
    ];
    for (i, perm) in permissions.iter().enumerate() {
        db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
            format!("INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-{}', '{}', 'desc', 'gescom')", i, perm))).await.unwrap();
        db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
            format!("INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-{}')", i))).await.unwrap();
    }

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User Admin', 'admin@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    // Seed products
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        INSERT INTO products (id, tenant_id, name, unit, purchase_price, selling_price, tax_rate, is_active, requires_prescription, created_at, updated_at)
        VALUES ('product-1', 'tenant-1', 'Smartphone Pro', 'unité', 500.0, 799.0, 0.20, 1, 0, '2026-05-20T10:00:00Z', '2026-05-20T10:00:00Z')
    ".to_string())).await.unwrap();

    // Create Stock Item
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        INSERT INTO stock_items (id, tenant_id, product_id, quantity, quantity_reserved, low_stock_threshold, updated_at)
        VALUES ('stock-1', 'tenant-1', 'product-1', 10.0, 0.0, 5.0, '2026-05-20T10:00:00Z')
    ".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db.clone()), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    // 1. Create Sale (2 units)
    let sale_payload = json!({
        "customer_name": "Jean Dupont",
        "customer_phone": "0606060606",
        "amount_paid": 2000.0,
        "payment_method": "cash",
        "notes": "Premium sale",
        "items": [
            {
                "product_id": "product-1",
                "quantity": 2.0,
                "unit_price": 799.0,
                "discount": 0.0
            }
        ]
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sales")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&sale_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let sale: Value = serde_json::from_slice(&bytes).unwrap();
    let sale_id = sale["id"].as_str().unwrap();
    assert_eq!(sale["customer_name"], "Jean Dupont");
    assert_eq!(sale["total"], 799.0 * 2.0); // subtotal + tax is calculated correctly or selling_price * qty
    assert_eq!(sale["status"], "completed");

    // 2. Verify Stock decremented to 8.0
    let stock_row = db.query_one(Statement::from_string(DatabaseBackend::Sqlite, "SELECT quantity FROM stock_items WHERE id = 'stock-1'".to_string())).await.unwrap().unwrap();
    let qty: f64 = stock_row.try_get_by_index(0).unwrap();
    assert_eq!(qty, 8.0);

    // 3. Read Sale Details
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/sales/{}", sale_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let fetched_sale: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(fetched_sale["id"], sale_id);
    assert_eq!(fetched_sale["items"].as_array().unwrap().len(), 1);

    // 4. List Sales
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/sales?customer_name=Jean")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let list_sales: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(list_sales["total"], 1);
    assert_eq!(list_sales["data"][0]["customer_name"], "Jean Dupont");

    // 5. Print Receipt
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/sales/{}/receipt", sale_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let receipt: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(receipt["receipt_number"], sale["receipt_number"]);
    assert_eq!(receipt["lines"].as_array().unwrap().len(), 1);

    // 6. Create another sale to trigger low stock (threshold is 5.0, selling 4 more units leaves quantity = 4.0)
    let sale_payload_low = json!({
        "customer_name": "Customer Low Stock",
        "amount_paid": 5000.0,
        "payment_method": "cash",
        "items": [
            {
                "product_id": "product-1",
                "quantity": 4.0,
                "unit_price": 799.0,
                "discount": 0.0
            }
        ]
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sales")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&sale_payload_low).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify stock is now 4.0
    let stock_row_low = db.query_one(Statement::from_string(DatabaseBackend::Sqlite, "SELECT quantity FROM stock_items WHERE id = 'stock-1'".to_string())).await.unwrap().unwrap();
    let qty_low: f64 = stock_row_low.try_get_by_index(0).unwrap();
    assert_eq!(qty_low, 4.0);

    // Verify low_stock alert triggered
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/alerts")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let alerts: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(alerts["total"], 1);
    assert_eq!(alerts["data"][0]["alert_type"], "low_stock");
    let alert_id = alerts["data"][0]["id"].as_str().unwrap();

    // 7. Mark alert as read
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/alerts/{}/read", alert_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let marked_alert: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(marked_alert["is_read"], true);

    // 8. Void the first sale (returns 2.0 units to stock)
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sales/{}/void", sale_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let voided_sale: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(voided_sale["status"], "voided");

    // Stock should go back up by 2.0 units (4.0 -> 6.0)
    let stock_row_void = db.query_one(Statement::from_string(DatabaseBackend::Sqlite, "SELECT quantity FROM stock_items WHERE id = 'stock-1'".to_string())).await.unwrap().unwrap();
    let qty_void: f64 = stock_row_void.try_get_by_index(0).unwrap();
    assert_eq!(qty_void, 6.0);
}

#[tokio::test]
async fn test_gescom_purchases_lifecycle() {
    let db = setup_test_db().await;

    // Seed tenant, role, permissions, user
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'My Tenant', 'pharmacy', 'own@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Tenant admin')".to_string())).await.unwrap();
    
    let permissions = vec![
        "can_create_purchase", "can_read_purchase", "can_update_purchase", "can_delete_purchase",
        "can_read_stock", "can_manage_stock"
    ];
    for (i, perm) in permissions.iter().enumerate() {
        db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
            format!("INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-{}', '{}', 'desc', 'gescom')", i, perm))).await.unwrap();
        db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
            format!("INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-{}')", i))).await.unwrap();
    }

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User Admin', 'admin@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    // Seed products
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        INSERT INTO products (id, tenant_id, name, unit, purchase_price, selling_price, tax_rate, is_active, requires_prescription, created_at, updated_at)
        VALUES ('product-1', 'tenant-1', 'Smartphone Pro', 'unité', 500.0, 799.0, 0.20, 1, 0, '2026-05-20T10:00:00Z', '2026-05-20T10:00:00Z')
    ".to_string())).await.unwrap();

    // Create Stock Item
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        INSERT INTO stock_items (id, tenant_id, product_id, quantity, quantity_reserved, low_stock_threshold, updated_at)
        VALUES ('stock-1', 'tenant-1', 'product-1', 5.0, 0.0, 2.0, '2026-05-20T10:00:00Z')
    ".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db.clone()), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    // 1. Create Purchase (10 units)
    let purchase_payload = json!({
        "supplier_name": "Alpha Distrib",
        "supplier_phone": "0102030405",
        "reference": "REF-998",
        "notes": "Restocking high priority",
        "items": [
            {
                "product_id": "product-1",
                "quantity": 10.0,
                "unit_cost": 450.0,
                "expiry_date": "2028-12-31",
                "batch_number": "BATCH-P100"
            }
        ]
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/purchases")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&purchase_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let purchase: Value = serde_json::from_slice(&bytes).unwrap();
    let purchase_id = purchase["id"].as_str().unwrap();
    assert_eq!(purchase["supplier_name"], "Alpha Distrib");
    assert_eq!(purchase["total"], 450.0 * 10.0);
    assert_eq!(purchase["status"], "received");

    // Stock should increase to 15.0 (5.0 + 10.0)
    let stock_row = db.query_one(Statement::from_string(DatabaseBackend::Sqlite, "SELECT quantity FROM stock_items WHERE id = 'stock-1'".to_string())).await.unwrap().unwrap();
    let qty: f64 = stock_row.try_get_by_index(0).unwrap();
    assert_eq!(qty, 15.0);

    // 2. Read Purchase Detail
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/purchases/{}", purchase_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let fetched_purchase: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(fetched_purchase["id"], purchase_id);

    // 3. List Purchases
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/purchases?supplier_name=Alpha")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let list_purchases: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(list_purchases["total"], 1);

    // 4. Cancel Purchase (reverts stock change by subtracting 10.0, back to 5.0)
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/purchases/{}/cancel", purchase_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let cancelled_purchase: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(cancelled_purchase["status"], "cancelled");

    // Stock should be back to 5.0
    let stock_row_cancel = db.query_one(Statement::from_string(DatabaseBackend::Sqlite, "SELECT quantity FROM stock_items WHERE id = 'stock-1'".to_string())).await.unwrap().unwrap();
    let qty_cancel: f64 = stock_row_cancel.try_get_by_index(0).unwrap();
    assert_eq!(qty_cancel, 5.0);
}

#[tokio::test]
async fn test_gescom_sync_logs() {
    let db = setup_test_db().await;

    // Seed tenant, role, permissions, user
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-1', 'My Tenant', 'pharmacy', 'own@tenant.com', 0)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO roles (id, tenant_id, name, description) VALUES ('role-1', 'tenant-1', 'admin', 'Tenant admin')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-sync-read', 'can_read_sync_log', 'Read sync', 'gescom')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-sync-write', 'can_manage_sync_log', 'Manage sync', 'gescom')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-sync-read')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO role_permissions (role_id, permission_id) VALUES ('role-1', 'perm-sync-write')".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('user-1', 'tenant-1', 'User Admin', 'admin@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO user_roles (user_id, role_id) VALUES ('user-1', 'role-1')".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db), config: config.clone() });
    let app = create_app(state);

    let token = create_token("user-1", "tenant-1", "admin", &config.jwt_secret);

    // 1. Create Sync Log
    let sync_payload = json!({
        "device_id": "DEVICE-ABC-123",
        "sync_type": "full",
        "status": "success",
        "records_pushed": 15,
        "records_pulled": 42
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sync/logs")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&sync_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let sync_log: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(sync_log["device_id"], "DEVICE-ABC-123");
    assert_eq!(sync_log["records_pushed"], 15);
    assert_eq!(sync_log["records_pulled"], 42);

    // 2. List Sync Logs
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/sync/logs?device_id=DEVICE-ABC-123")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let list_sync: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(list_sync["total"], 1);
    assert_eq!(list_sync["data"][0]["device_id"], "DEVICE-ABC-123");
}

#[tokio::test]
async fn test_cross_tenant_restrictions_gescom() {
    let db = setup_test_db().await;

    // Seed system tenant and a regular tenant
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('system-tenant', 'System Admin', 'both', 'sys@tenant.com', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO tenants (id, name, business_type, email, is_system) VALUES ('tenant-2', 'Regular Tenant', 'pharmacy', 'reg@tenant.com', 0)".to_string())).await.unwrap();

    // Create system role and user
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO roles (id, tenant_id, name, description) VALUES ('sys-role', 'system-tenant', 'admin', 'System Admin Role')".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO users (id, tenant_id, name, email, password_hash, is_active) VALUES ('sys-user', 'system-tenant', 'Sys Admin', 'sys@tenant.com', 'hash', 1)".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO user_roles (user_id, role_id) VALUES ('sys-user', 'sys-role')".to_string())).await.unwrap();

    // Permissions needed
    let permissions = vec![
        "can_create_sale", "can_read_sale", 
        "can_access_other_tenant_for_creation"
    ];
    for (i, perm) in permissions.iter().enumerate() {
        db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
            format!("INSERT INTO permissions (id, name, description, model_group) VALUES ('perm-{}', '{}', 'desc', 'system')", i, perm))).await.unwrap();
    }

    // Give can_create_sale to the role, but NOT can_access_other_tenant_for_creation initially
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO role_permissions (role_id, permission_id) VALUES ('sys-role', 'perm-0')".to_string())).await.unwrap(); // can_create_sale
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO role_permissions (role_id, permission_id) VALUES ('sys-role', 'perm-1')".to_string())).await.unwrap(); // can_read_sale

    // Seed product under tenant-2
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        INSERT INTO products (id, tenant_id, name, unit, purchase_price, selling_price, tax_rate, is_active, requires_prescription, created_at, updated_at)
        VALUES ('product-t2', 'tenant-2', 'Product of Tenant 2', 'unité', 10.0, 15.0, 0.18, 1, 0, '2026-05-20T10:00:00Z', '2026-05-20T10:00:00Z')
    ".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        INSERT INTO stock_items (id, tenant_id, product_id, quantity, quantity_reserved, low_stock_threshold, updated_at)
        VALUES ('stock-t2', 'tenant-2', 'product-t2', 10.0, 0.0, 2.0, '2026-05-20T10:00:00Z')
    ".to_string())).await.unwrap();

    let config = Config::default();
    let state = Arc::new(AppState { db: Some(db.clone()), config: config.clone() });
    let app = create_app(state);

    // Call as sys-user but targeting tenant-2
    let token = create_token("sys-user", "system-tenant", "admin", &config.jwt_secret);

    let cross_payload = json!({
        "tenant_id": "tenant-2",
        "customer_name": "Cross Tenant Cust",
        "amount_paid": 100.0,
        "payment_method": "cash",
        "items": [
            {
                "product_id": "product-t2",
                "quantity": 1.0,
                "unit_price": 15.0,
                "discount": 0.0
            }
        ]
    });

    // 1. Should fail with 401 Unauthorized because sys-user lacks `can_access_other_tenant_for_creation`
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sales")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&cross_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // 2. Grant can_access_other_tenant_for_creation to the role
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, 
        "INSERT INTO role_permissions (role_id, permission_id) VALUES ('sys-role', 'perm-2')".to_string())).await.unwrap(); // can_access_other_tenant_for_creation

    // 3. Should succeed now
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sales")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&cross_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
