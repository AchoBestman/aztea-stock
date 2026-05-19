use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, PaginatorTrait};
use crate::{
    errors::ApiError,
    models::{user_role, role_permission, permission, role}
};

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

/// Enforces that a user has a specific permission, returning an ApiError::Unauthorized if not.
pub async fn require_permission(
    db: &sea_orm::DatabaseConnection,
    user_id: &str,
    permission_name: &str,
) -> Result<(), ApiError> {
    match check_permission(db, user_id, permission_name).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(ApiError::Unauthorized(format!(
            "Vous n'avez pas la permission requise : {}",
            permission_name
        ))),
        Err(e) => Err(ApiError::Internal(e.to_string())),
    }
}
