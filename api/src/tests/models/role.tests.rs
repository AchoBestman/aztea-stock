use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
use crate::repositories::role_repository::RoleRepository;

#[tokio::test]
async fn test_role_model_crud() {
    // Connect to a local temporary SQLite file to ensure all pool connections share the same database
    let db_url = "sqlite://test_role_model.db?mode=rwc";
    let db = Database::connect(db_url).await.unwrap();
    
    // Setup temporary schema (drop first to ensure a clean state)
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "DROP TABLE IF EXISTS roles;".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "DROP TABLE IF EXISTS tenants;".to_string())).await.unwrap();

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

    let tenant_id = "tenant-1";
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "INSERT INTO tenants (id, name, business_type, email) VALUES ('tenant-1', 'Test Tenant', 'both', 'test@tenant.com')".to_string())).await.unwrap();

    // 1. Create a Role
    let role = RoleRepository::create(&db, "role-1", tenant_id, "Manager", Some("Gestionnaire".to_string())).await.unwrap();
    assert_eq!(role.name, "Manager");
    assert_eq!(role.description, Some("Gestionnaire".to_string()));
    assert_eq!(role.tenant_id, tenant_id);

    // 2. Check existence
    let exists = RoleRepository::exists_by_name(&db, "Manager", tenant_id).await.unwrap();
    assert!(exists);
    let not_exists = RoleRepository::exists_by_name(&db, "Cashier", tenant_id).await.unwrap();
    assert!(!not_exists);

    // 3. Find by ID
    let found = RoleRepository::find_by_id(&db, &role.id, tenant_id).await.unwrap().unwrap();
    assert_eq!(found.name, "Manager");

    // 4. Update
    let updated = RoleRepository::update(&db, &role.id, tenant_id, "Senior Manager", Some("Gestionnaire Senior".to_string())).await.unwrap();
    assert_eq!(updated.name, "Senior Manager");

    // 5. List
    let list = RoleRepository::find_all_by_tenant(&db, tenant_id).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Senior Manager");

    // 6. Delete
    let deleted = RoleRepository::delete(&db, &role.id, tenant_id).await.unwrap();
    assert_eq!(deleted.rows_affected, 1);
    let list_after = RoleRepository::find_all_by_tenant(&db, tenant_id).await.unwrap();
    assert_eq!(list_after.len(), 0);

    // Cleanup
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "DROP TABLE IF EXISTS roles;".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "DROP TABLE IF EXISTS tenants;".to_string())).await.unwrap();
    
    drop(db);
    let _ = std::fs::remove_file("test_role_model.db");
}
