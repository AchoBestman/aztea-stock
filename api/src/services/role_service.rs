use sea_orm::{DatabaseConnection, PaginatorTrait};
use validator::Validate;
use crate::{
    errors::ApiError,
    repositories::role_repository::RoleRepository,
    dtos::{
        create_role_dto::CreateRolePayload,
        update_role_dto::UpdateRolePayload,
        response_role_dto::{RoleResponse, DeleteRoleResponse}
    },
    schemas::role_schema::RoleValidationSchema
};

pub struct RoleService;

impl RoleService {
    pub async fn list_roles(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        filter_tenant_id: Option<String>,
        name_query: Option<String>,
    ) -> Result<Vec<RoleResponse>, ApiError> {
        // 1. Fetch caller's tenant
        let caller_tenant = crate::repositories::tenant_repository::TenantRepository::find_by_id(db, caller_tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Tenant de l'utilisateur introuvable".to_string()))?;

        // 2. Determine target tenant filter based on permission guard
        let target_tenant = if let Some(ref f_t_id) = filter_tenant_id {
            crate::utils::auth::require_tenant_access(db, caller_tenant_id, f_t_id).await?;
            Some(f_t_id.as_str())
        } else if caller_tenant.is_system {
            // System tenant users see all roles if no filter is specified
            None
        } else {
            // Regular users are locked to their own tenant
            Some(caller_tenant_id)
        };

        let models = RoleRepository::find_all_filtered(db, target_tenant, name_query.as_deref()).await?;
        let is_sys_sa = Self::is_system_super_admin(db, caller_user_id, caller_tenant_id).await.unwrap_or(false);

        let mut dtos = Vec::new();
        for m in models {
            // Only show the "Super Admin" role to system Super Admin callers
            if m.name == "Super Admin" && !is_sys_sa {
                continue;
            }
            dtos.push(RoleResponse {
                id: m.id,
                tenant_id: m.tenant_id,
                name: m.name,
                description: m.description,
                permissions: None,
            });
        }
        Ok(dtos)
    }

    pub async fn get_role(
        db: &DatabaseConnection,
        caller_user_id: &str,
        id: &str,
        caller_tenant_id: &str,
    ) -> Result<RoleResponse, ApiError> {
        // Fetch globally first
        let m = RoleRepository::find_by_id_global(db, id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Rôle introuvable".to_string()))?;

        // Multi-tenant guard
        crate::utils::auth::require_tenant_access(db, caller_tenant_id, &m.tenant_id).await?;

        // If the role is the system Super Admin, only allow system Super Admin users to access
        if m.name == "Super Admin" {
            let is_sys_sa = Self::is_system_super_admin(db, caller_user_id, caller_tenant_id).await?;
            if !is_sys_sa {
                return Err(ApiError::NotFound("Rôle introuvable".to_string()));
            }
        }

        // Fetch associated permissions for this role
        let perms = Self::list_role_permissions(db, caller_user_id, id, caller_tenant_id).await?;

        Ok(RoleResponse {
            id: m.id,
            tenant_id: m.tenant_id,
            name: m.name,
            description: m.description,
            permissions: Some(perms),
        })
    }

    pub async fn create_role(
        db: &DatabaseConnection,
        caller_tenant_id: &str,
        payload: CreateRolePayload,
    ) -> Result<RoleResponse, ApiError> {
        // Validation schema
        let validator = RoleValidationSchema {
            name: &payload.name,
            description: payload.description.as_deref(),
        };
        validator.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

        // Determine final tenant id
        let final_tenant_id = if let Some(ref t_id) = payload.tenant_id {
            crate::utils::auth::require_tenant_access(db, caller_tenant_id, t_id).await?;
            t_id.clone()
        } else {
            caller_tenant_id.to_string()
        };

        // Business validation: reject 'Super Admin' name via API
        if payload.name.trim() == "Super Admin" {
            return Err(ApiError::BadRequest("Le rôle 'Super Admin' ne peut être créé que via le système.".to_string()));
        }

        // Business validation: uniqueness of name within the final tenant
        if RoleRepository::exists_by_name(db, &payload.name, &final_tenant_id).await? {
            return Err(ApiError::BadRequest("Un rôle avec ce nom existe déjà pour ce tenant.".to_string()));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let m = RoleRepository::create(db, &id, &final_tenant_id, &payload.name, payload.description).await?;

        Ok(RoleResponse {
            id: m.id,
            tenant_id: m.tenant_id,
            name: m.name,
            description: m.description,
            permissions: None,
        })
    }

    pub async fn update_role(
        db: &DatabaseConnection,
        id: &str,
        caller_tenant_id: &str,
        payload: UpdateRolePayload,
    ) -> Result<RoleResponse, ApiError> {
        // Validation schema
        let validator = RoleValidationSchema {
            name: &payload.name,
            description: payload.description.as_deref(),
        };
        validator.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

        // Fetch globally first
        let role = RoleRepository::find_by_id_global(db, id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Rôle introuvable".to_string()))?;

        // Business validation: reject renaming or modifying Super Admin
        if role.name == "Super Admin" {
            return Err(ApiError::BadRequest("Le rôle 'Super Admin' ne peut pas être modifié.".to_string()));
        }
        if payload.name.trim() == "Super Admin" {
            return Err(ApiError::BadRequest("Un rôle ne peut pas être renommé ou défini en 'Super Admin'.".to_string()));
        }

        // Multi-tenant guard
        crate::utils::auth::require_tenant_access(db, caller_tenant_id, &role.tenant_id).await?;

        // Business validation: uniqueness of name within its tenant
        if RoleRepository::exists_by_name_exclude(db, &payload.name, &role.tenant_id, id).await? {
            return Err(ApiError::BadRequest("Un rôle avec ce nom existe déjà pour ce tenant.".to_string()));
        }

        let m = RoleRepository::update(db, id, &role.tenant_id, &payload.name, payload.description).await?;

        Ok(RoleResponse {
            id: m.id,
            tenant_id: m.tenant_id,
            name: m.name,
            description: m.description,
            permissions: None,
        })
    }

    pub async fn delete_role(
        db: &DatabaseConnection,
        id: &str,
        caller_tenant_id: &str,
    ) -> Result<DeleteRoleResponse, ApiError> {
        use crate::models::user_role;
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait};

        // Fetch globally first
        let role = RoleRepository::find_by_id_global(db, id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Rôle introuvable".to_string()))?;

        // Business validation: reject deleting Super Admin
        if role.name == "Super Admin" {
            return Err(ApiError::BadRequest("Le rôle 'Super Admin' ne peut pas être supprimé.".to_string()));
        }

        // Multi-tenant guard
        crate::utils::auth::require_tenant_access(db, caller_tenant_id, &role.tenant_id).await?;

        let user_role_count = user_role::Entity::find()
            .filter(user_role::Column::RoleId.eq(id))
            .count(db)
            .await?;

        if user_role_count > 0 {
            return Err(ApiError::BadRequest(
                "Ce rôle ne peut pas être supprimé car il est actuellement attribué à un ou plusieurs utilisateurs.".to_string()
            ));
        }

        let result = RoleRepository::delete(db, id, &role.tenant_id).await?;
        if result.rows_affected == 0 {
            return Err(ApiError::NotFound("Rôle introuvable".to_string()));
        }

        Ok(DeleteRoleResponse {
            success: true,
            message: "Rôle supprimé avec succès.".to_string(),
        })
    }

    pub async fn sync_role_permissions(
        db: &DatabaseConnection,
        role_id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
        permission_ids: Vec<String>,
    ) -> Result<(), ApiError> {
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set, TransactionTrait};
        use crate::models::{role_permission, permission};

        // 1. Fetch role globally to ensure it exists
        let role = RoleRepository::find_by_id_global(db, role_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Rôle introuvable".to_string()))?;

        // 2. Multi-tenant guard
        crate::utils::auth::require_tenant_access(db, caller_tenant_id, &role.tenant_id).await?;

        // 3. Prevent modifying permissions of the "Super Admin" role unless they are a system super admin
        if role.name == "Super Admin" {
            let is_sys_sa = Self::is_system_super_admin(db, caller_user_id, caller_tenant_id).await?;
            if !is_sys_sa {
                return Err(ApiError::BadRequest("Seul un Super Admin du tenant système peut modifier les permissions du rôle 'Super Admin'.".to_string()));
            }
        }

        // 4. Validate that all provided permission_ids exist in the database
        if !permission_ids.is_empty() {
            let existing_count = permission::Entity::find()
                .filter(permission::Column::Id.is_in(permission_ids.clone()))
                .count(db)
                .await?;

            if existing_count != permission_ids.len() as u64 {
                return Err(ApiError::BadRequest(
                    "Une ou plusieurs permissions spécifiées sont introuvables ou invalides.".to_string()
                ));
            }
        }

        // 5. Perform the sync inside a transaction to ensure atomicity
        let role_id_owned = role_id.to_string();
        db.transaction::<_, (), ApiError>(move |txn| {
            let r_id = role_id_owned.clone();
            let p_ids = permission_ids.clone();
            Box::pin(async move {
                // Delete all existing permissions for this role
                role_permission::Entity::delete_many()
                    .filter(role_permission::Column::RoleId.eq(&r_id))
                    .exec(txn)
                    .await?;

                // Insert the new permissions
                for perm_id in p_ids {
                    let active_rp = role_permission::ActiveModel {
                        role_id: Set(r_id.clone()),
                        permission_id: Set(perm_id),
                    };
                    active_rp.insert(txn).await?;
                }

                Ok(())
            })
        })
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Connection(db_err) => ApiError::Database(db_err),
            sea_orm::TransactionError::Transaction(api_err) => api_err,
        })?;

        Ok(())
    }

    pub async fn list_role_permissions(
        db: &DatabaseConnection,
        caller_user_id: &str,
        role_id: &str,
        caller_tenant_id: &str,
    ) -> Result<Vec<crate::services::permission_service::PermissionResponse>, ApiError> {
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
        use crate::models::{role_permission, permission};

        // 1. Fetch role globally first to ensure it exists
        let role = RoleRepository::find_by_id_global(db, role_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Rôle introuvable".to_string()))?;

        // 2. Multi-tenant guard
        crate::utils::auth::require_tenant_access(db, caller_tenant_id, &role.tenant_id).await?;

        // 3. If it's the system Super Admin role, only allow system Super Admin users to view
        if role.name == "Super Admin" {
            let is_sys_sa = Self::is_system_super_admin(db, caller_user_id, caller_tenant_id).await?;
            if !is_sys_sa {
                return Err(ApiError::NotFound("Rôle introuvable".to_string()));
            }

            let all_perms = permission::Entity::find().all(db).await?;
            let response = all_perms
                .into_iter()
                .map(|p| crate::services::permission_service::PermissionResponse {
                    id: p.id,
                    name: p.name,
                    description: p.description,
                })
                .collect();
            return Ok(response);
        }

        // 4. Fetch permissions associated with this role
        let role_perms = role_permission::Entity::find()
            .filter(role_permission::Column::RoleId.eq(role_id))
            .all(db)
            .await?;

        if role_perms.is_empty() {
            return Ok(Vec::new());
        }

        let perm_ids: Vec<String> = role_perms.into_iter().map(|rp| rp.permission_id).collect();

        let perms = permission::Entity::find()
            .filter(permission::Column::Id.is_in(perm_ids))
            .all(db)
            .await?;

        let response = perms
            .into_iter()
            .map(|p| crate::services::permission_service::PermissionResponse {
                id: p.id,
                name: p.name,
                description: p.description,
            })
            .collect();

        Ok(response)
    }

    pub async fn is_system_super_admin(
        db: &DatabaseConnection,
        user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<bool, ApiError> {
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
        use crate::models::{user_role, role};

        // 1. Fetch tenant
        let caller_tenant = crate::repositories::tenant_repository::TenantRepository::find_by_id(db, caller_tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Tenant de l'utilisateur introuvable".to_string()))?;

        if !caller_tenant.is_system {
            return Ok(false);
        }

        // 2. Fetch user's roles
        let user_roles = user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        let role_ids: Vec<String> = user_roles.into_iter().map(|ur| ur.role_id).collect();
        if role_ids.is_empty() {
            return Ok(false);
        }

        let roles = role::Entity::find()
            .filter(role::Column::Id.is_in(role_ids))
            .all(db)
            .await?;

        let is_super_admin = roles.iter().any(|r| r.name == "Super Admin");
        Ok(is_super_admin)
    }
}
