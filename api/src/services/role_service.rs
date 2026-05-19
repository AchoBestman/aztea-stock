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
        tenant_id: &str,
    ) -> Result<Vec<RoleResponse>, ApiError> {
        let models = RoleRepository::find_all_by_tenant(db, tenant_id).await?;
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
        tenant_id: &str,
    ) -> Result<RoleResponse, ApiError> {
        let m = RoleRepository::find_by_id(db, id, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Rôle introuvable".to_string()))?;
        Ok(RoleResponse {
            id: m.id,
            tenant_id: m.tenant_id,
            name: m.name,
            description: m.description,
        })
    }

    pub async fn create_role(
        db: &DatabaseConnection,
        tenant_id: &str,
        payload: CreateRolePayload,
    ) -> Result<RoleResponse, ApiError> {
        // Validation schema
        let validator = RoleValidationSchema {
            name: &payload.name,
            description: payload.description.as_deref(),
        };
        validator.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

        // Business validation: uniqueness of name
        if RoleRepository::exists_by_name(db, &payload.name, tenant_id).await? {
            return Err(ApiError::BadRequest("Un rôle avec ce nom existe déjà pour ce tenant.".to_string()));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let m = RoleRepository::create(db, &id, tenant_id, &payload.name, payload.description).await?;

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
        tenant_id: &str,
        payload: UpdateRolePayload,
    ) -> Result<RoleResponse, ApiError> {
        // Validation schema
        let validator = RoleValidationSchema {
            name: &payload.name,
            description: payload.description.as_deref(),
        };
        validator.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

        // Business validation: check if role exists
        if !RoleRepository::exists_by_id(db, id, tenant_id).await? {
            return Err(ApiError::NotFound("Rôle introuvable".to_string()));
        }

        // Business validation: uniqueness of name if changed
        if RoleRepository::exists_by_name_exclude(db, &payload.name, tenant_id, id).await? {
            return Err(ApiError::BadRequest("Un rôle avec ce nom existe déjà pour ce tenant.".to_string()));
        }

        let m = RoleRepository::update(db, id, tenant_id, &payload.name, payload.description).await?;

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
        tenant_id: &str,
    ) -> Result<DeleteRoleResponse, ApiError> {
        use crate::models::user_role;
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait};

        let user_role_count = user_role::Entity::find()
            .filter(user_role::Column::RoleId.eq(id))
            .count(db)
            .await?;

        if user_role_count > 0 {
            return Err(ApiError::BadRequest(
                "Ce rôle ne peut pas être supprimé car il est actuellement attribué à un ou plusieurs utilisateurs.".to_string()
            ));
        }

        let result = RoleRepository::delete(db, id, tenant_id).await?;
        if result.rows_affected == 0 {
            return Err(ApiError::NotFound("Rôle introuvable".to_string()));
        }

        Ok(DeleteRoleResponse {
            success: true,
            message: "Rôle supprimé avec succès.".to_string(),
        })
    }
}
