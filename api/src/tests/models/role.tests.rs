use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use crate::repositories::role_repository::RoleRepository;

#[tokio::test]
async fn test_role_model_crud() {
    let db = crate::tests::helpers::setup_test_db().await;

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

}
