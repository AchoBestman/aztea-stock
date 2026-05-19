use sea_orm::DatabaseConnection;
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
            if f_t_id != caller_tenant_id && !caller_tenant.is_system {
                return Err(ApiError::Unauthorized(
                    "Vous n'êtes pas autorisé à filtrer les données pour un autre tenant.".to_string()
                ));
            }
            Some(f_t_id.as_str())
        } else if caller_tenant.is_system {
            // System tenant users see all roles if no filter is specified
            None
        } else {
            // Regular users are locked to their own tenant
            Some(caller_tenant_id)
        };

        let models = RoleRepository::find_all_filtered(db, target_tenant, name_query.as_deref()).await?;
        let dtos = models
            .into_iter()
            .map(|m| RoleResponse {
                id: m.id,
                tenant_id: m.tenant_id,
                name: m.name,
                description: m.description,
            })
            .collect();
        Ok(dtos)
    }

    pub async fn get_role(
        db: &DatabaseConnection,
        id: &str,
        caller_tenant_id: &str,
    ) -> Result<RoleResponse, ApiError> {
        // Fetch globally first
        let m = RoleRepository::find_by_id_global(db, id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Rôle introuvable".to_string()))?;

        // Multi-tenant guard
        if m.tenant_id != caller_tenant_id {
            let caller_tenant = crate::repositories::tenant_repository::TenantRepository::find_by_id(db, caller_tenant_id)
                .await?
                .ok_or_else(|| ApiError::NotFound("Tenant de l'utilisateur introuvable".to_string()))?;
            if !caller_tenant.is_system {
                return Err(ApiError::Unauthorized(
                    "Vous n'êtes pas autorisé à accéder aux données d'un autre tenant.".to_string()
                ));
            }
        }

        Ok(RoleResponse {
            id: m.id,
            tenant_id: m.tenant_id,
            name: m.name,
            description: m.description,
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
            if t_id != caller_tenant_id {
                let caller_tenant = crate::repositories::tenant_repository::TenantRepository::find_by_id(db, caller_tenant_id)
                    .await?
                    .ok_or_else(|| ApiError::NotFound("Tenant de l'utilisateur introuvable".to_string()))?;
                if !caller_tenant.is_system {
                    return Err(ApiError::Unauthorized(
                        "Vous n'êtes pas autorisé à créer des rôles pour un autre tenant.".to_string()
                    ));
                }
            }
            t_id.clone()
        } else {
            caller_tenant_id.to_string()
        };

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

        // Multi-tenant guard
        if role.tenant_id != caller_tenant_id {
            let caller_tenant = crate::repositories::tenant_repository::TenantRepository::find_by_id(db, caller_tenant_id)
                .await?
                .ok_or_else(|| ApiError::NotFound("Tenant de l'utilisateur introuvable".to_string()))?;
            if !caller_tenant.is_system {
                return Err(ApiError::Unauthorized(
                    "Vous n'êtes pas autorisé à modifier les données d'un autre tenant.".to_string()
                ));
            }
        }

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

        // Multi-tenant guard
        if role.tenant_id != caller_tenant_id {
            let caller_tenant = crate::repositories::tenant_repository::TenantRepository::find_by_id(db, caller_tenant_id)
                .await?
                .ok_or_else(|| ApiError::NotFound("Tenant de l'utilisateur introuvable".to_string()))?;
            if !caller_tenant.is_system {
                return Err(ApiError::Unauthorized(
                    "Vous n'êtes pas autorisé à supprimer les données d'un autre tenant.".to_string()
                ));
            }
        }

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
}
