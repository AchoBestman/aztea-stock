use crate::{
    errors::ApiError,
    models::{permission, role, role_permission, tenant, user_role},
};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

/// Rôles et noms de permissions effectifs pour un utilisateur (Super Admin → toutes les permissions).
pub async fn fetch_user_roles_and_permissions(
    db: &sea_orm::DatabaseConnection,
    user_id: &str,
) -> Result<(Vec<String>, Vec<String>), sea_orm::DbErr> {
    let user_roles = user_role::Entity::find()
        .filter(user_role::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    let role_ids: Vec<String> = user_roles.into_iter().map(|ur| ur.role_id).collect();
    if role_ids.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    let roles = role::Entity::find()
        .filter(role::Column::Id.is_in(role_ids.clone()))
        .all(db)
        .await?;
    let role_names: Vec<String> = roles.iter().map(|r| r.name.clone()).collect();

    if roles.iter().any(|r| r.name == "Super Admin") {
        let all_perms = permission::Entity::find().all(db).await?;
        let perm_names: Vec<String> = all_perms.into_iter().map(|p| p.name).collect();
        return Ok((role_names, perm_names));
    }

    let role_perms = role_permission::Entity::find()
        .filter(role_permission::Column::RoleId.is_in(role_ids))
        .all(db)
        .await?;
    let perm_ids: Vec<String> = role_perms.into_iter().map(|rp| rp.permission_id).collect();
    if perm_ids.is_empty() {
        return Ok((role_names, Vec::new()));
    }

    let permissions = permission::Entity::find()
        .filter(permission::Column::Id.is_in(perm_ids))
        .all(db)
        .await?;
    let perm_names: Vec<String> = permissions.into_iter().map(|p| p.name).collect();
    Ok((role_names, perm_names))
}

/// Check if a user has a specific permission.
/// Returns true if the user has the required permission or has the "Super Admin" role.
pub async fn check_permission(
    db: &sea_orm::DatabaseConnection,
    user_id: &str,
    permission_name: &str,
) -> Result<bool, sea_orm::DbErr> {
    // 1. Get user roles
    let user_roles = user_role::Entity::find()
        .filter(user_role::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    let role_ids: Vec<String> = user_roles.into_iter().map(|ur| ur.role_id).collect();
    if role_ids.is_empty() {
        return Ok(false);
    }

    // 2. Check if user has "Super Admin" role (automatically grants all permissions)
    let roles = role::Entity::find()
        .filter(role::Column::Id.is_in(role_ids.clone()))
        .all(db)
        .await?;
    if roles.iter().any(|r| r.name == "Super Admin") {
        return Ok(true);
    }

    // 3. Find if required permission is present in any of user's roles
    let role_perms = role_permission::Entity::find()
        .filter(role_permission::Column::RoleId.is_in(role_ids))
        .all(db)
        .await?;

    let perm_ids: Vec<String> = role_perms.into_iter().map(|rp| rp.permission_id).collect();
    if perm_ids.is_empty() {
        return Ok(false);
    }

    let count = permission::Entity::find()
        .filter(permission::Column::Id.is_in(perm_ids))
        .filter(permission::Column::Name.eq(permission_name))
        .count(db)
        .await?;

    Ok(count > 0)
}

/// Accès administration globale (licences, abonnements cross-tenant) : tenant système ou Super Admin.
pub async fn assert_system_admin_access(
    db: &sea_orm::DatabaseConnection,
    user_id: &str,
    caller_tenant_id: &str,
) -> Result<(), ApiError> {
    if crate::services::role_service::RoleService::is_system_super_admin(
        db,
        user_id,
        caller_tenant_id,
    )
    .await?
    {
        return Ok(());
    }

    let caller = tenant::Entity::find_by_id(caller_tenant_id)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Tenant de l'opérateur introuvable".to_string()))?;

    if caller.is_system {
        return Ok(());
    }

    Err(ApiError::Forbidden(
        "Action réservée au tenant système ou au rôle Super Admin.".to_string(),
    ))
}

/// Enforces that a user has a specific permission, returning an ApiError::Forbidden if not.
pub async fn require_permission(
    db: &sea_orm::DatabaseConnection,
    user_id: &str,
    permission_name: &str,
) -> Result<(), ApiError> {
    match check_permission(db, user_id, permission_name).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(ApiError::Forbidden(format!(
            "Vous n'avez pas la permission requise : {}",
            permission_name
        ))),
        Err(e) => Err(ApiError::Internal(e.to_string())),
    }
}

/// Enforces multi-tenant isolation.
/// If target_tenant_id is different from caller_tenant_id, it verifies that the caller_tenant has `is_system = true`.
/// If not, it returns ApiError::Unauthorized.
pub async fn require_tenant_access(
    db: &sea_orm::DatabaseConnection,
    caller_tenant_id: &str,
    target_tenant_id: &str,
    user_id: &str,
    action_type: &str, // "read" | "create" | "update" | "delete"
) -> Result<(), ApiError> {
    if target_tenant_id != caller_tenant_id {
        let caller_tenant = crate::repositories::tenant_repository::TenantRepository::find_by_id(
            db,
            caller_tenant_id,
        )
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Tenant de l'utilisateur introuvable".to_string()))?;

        if !caller_tenant.is_system {
            return Err(ApiError::Forbidden(
                "Vous n'êtes pas autorisé à modifier ou accéder aux données d'un autre tenant."
                    .to_string(),
            ));
        }

        // It is a system tenant, check specific cross-tenant permission
        let required_permission = match action_type {
            "read" => "can_access_other_tenant_for_edition", // "edition" used for read/list as per requirements
            "create" => "can_access_other_tenant_for_creation",
            "update" => "can_access_other_tenant_for_updating",
            "delete" => "can_access_other_tenant_for_deleting",
            _ => {
                return Err(ApiError::Internal(
                    "Action type invalide pour cross-tenant".to_string(),
                ));
            }
        };

        crate::utils::auth::require_permission(db, user_id, required_permission).await?;
    }
    Ok(())
}
