use crate::models::role::Role;
use sqlx::{AnyPool, Executor};

#[tokio::test]
async fn test_role_model_crud() {
    // Connect to a local temporary SQLite file to ensure all pool connections share the same database
    let db_url = "sqlite://test_role_model.db?mode=rwc";
    let pool = AnyPool::connect(db_url).await.unwrap();
    
    // Setup temporary schema (drop first to ensure a clean state)
    pool.execute("DROP TABLE IF EXISTS roles;").await.unwrap();
    pool.execute("DROP TABLE IF EXISTS tenants;").await.unwrap();

    pool.execute("
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
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
        );
    ").await.unwrap();
    
    pool.execute("
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
    ").await.unwrap();

    let tenant_id = "tenant-1";
    pool.execute("INSERT INTO tenants (id, name, business_type, email) VALUES ('tenant-1', 'Test Tenant', 'both', 'test@tenant.com')").await.unwrap();

    // 1. Create a Role
    let role = Role::create(&pool, tenant_id, "Manager", Some("Gestionnaire")).await.unwrap();
    assert_eq!(role.name, "Manager");
    assert_eq!(role.description, Some("Gestionnaire".to_string()));
    assert_eq!(role.tenant_id, tenant_id);

    // 2. Check existence
    let exists = Role::exists_by_name(&pool, tenant_id, "Manager").await.unwrap();
    assert!(exists);
    let not_exists = Role::exists_by_name(&pool, tenant_id, "Cashier").await.unwrap();
    assert!(!not_exists);

    // 3. Find by ID
    let found = Role::find_by_id(&pool, tenant_id, &role.id).await.unwrap().unwrap();
    assert_eq!(found.name, "Manager");

    // 4. Update
    let updated = Role::update(&pool, &role.id, tenant_id, "Senior Manager", Some("Gestionnaire Senior")).await.unwrap();
    assert_eq!(updated.name, "Senior Manager");

    // 5. List
    let list = Role::list_by_tenant(&pool, tenant_id).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Senior Manager");

    // 6. Delete
    let deleted = Role::delete(&pool, &role.id, tenant_id).await.unwrap();
    assert!(deleted);
    let list_after = Role::list_by_tenant(&pool, tenant_id).await.unwrap();
    assert_eq!(list_after.len(), 0);

    // Cleanup
    pool.execute("DROP TABLE IF EXISTS roles;").await.unwrap();
    pool.execute("DROP TABLE IF EXISTS tenants;").await.unwrap();
    
    drop(pool);
    let _ = std::fs::remove_file("test_role_model.db");
}
