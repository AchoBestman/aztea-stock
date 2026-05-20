use sea_orm::{DatabaseConnection, PaginatorTrait};
use validator::Validate;
use crate::{
    errors::ApiError,
    repositories::category_repository::CategoryRepository,
    dtos::category_dto::{CreateCategoryPayload, UpdateCategoryPayload, CategoryResponse},
};

pub struct CategoryService;

impl CategoryService {
    pub async fn create_category(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        target_tenant_id: Option<String>,
        payload: CreateCategoryPayload,
    ) -> Result<CategoryResponse, ApiError> {
        payload.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

        let final_tenant_id = if let Some(t_id) = target_tenant_id {
            crate::utils::auth::require_tenant_access(db, caller_tenant_id, &t_id, caller_user_id, "create").await?;
            t_id
        } else {
            caller_tenant_id.to_string()
        };

        if let Some(ref pid) = payload.parent_id {
            let _parent = CategoryRepository::find_by_id(db, pid, &final_tenant_id)
                .await?
                .ok_or_else(|| ApiError::BadRequest("Catégorie parente introuvable.".to_string()))?;
        }

        let is_active = payload.is_active.unwrap_or(true);
        let id = uuid::Uuid::new_v4().to_string();
        let created = CategoryRepository::create(
            db,
            &id,
            &final_tenant_id,
            &payload.name,
            payload.description,
            payload.color,
            payload.icon,
            payload.parent_id,
            is_active,
        ).await?;

        Self::map_to_response(db, created).await
    }

    pub async fn get_category(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<CategoryResponse, ApiError> {
        // Find category across tenant bounds using just ID (since ID is globally unique)
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
        use crate::models::category;

        let model = category::Entity::find()
            .filter(category::Column::Id.eq(id))
            .filter(category::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(|e| ApiError::Database(e))?
            .ok_or_else(|| ApiError::NotFound("Catégorie introuvable".to_string()))?;

        // Guard
        crate::utils::auth::require_tenant_access(db, caller_tenant_id, &model.tenant_id, caller_user_id, "read").await?;

        Self::map_to_response(db, model).await
    }

    pub async fn list_categories(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        target_tenant_id: Option<String>,
        parent_id: Option<String>,
        is_active: Option<bool>,
        params: crate::utils::pagination::PaginationParams,
    ) -> Result<crate::utils::pagination::PaginatedResponse<CategoryResponse>, ApiError> {
        let final_tenant_id = if let Some(t_id) = target_tenant_id {
            crate::utils::auth::require_tenant_access(db, caller_tenant_id, &t_id, caller_user_id, "read").await?;
            t_id
        } else {
            caller_tenant_id.to_string()
        };

        let paginated_models = CategoryRepository::find_all_paginated(
            db,
            &final_tenant_id,
            parent_id,
            is_active,
            params,
        ).await?;

        let data = Self::map_to_response_list(db, paginated_models.data).await?;

        Ok(crate::utils::pagination::PaginatedResponse {
            data,
            total: paginated_models.total,
            page: paginated_models.page,
            per_page: paginated_models.per_page,
            total_pages: paginated_models.total_pages,
        })
    }

    pub async fn update_category(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
        payload: UpdateCategoryPayload,
    ) -> Result<CategoryResponse, ApiError> {
        payload.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
        use crate::models::category;

        let model = category::Entity::find()
            .filter(category::Column::Id.eq(id))
            .filter(category::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(|e| ApiError::Database(e))?
            .ok_or_else(|| ApiError::NotFound("Catégorie introuvable".to_string()))?;

        crate::utils::auth::require_tenant_access(db, caller_tenant_id, &model.tenant_id, caller_user_id, "update").await?;

        if let Some(ref pid) = payload.parent_id {
            if pid == id {
                return Err(ApiError::BadRequest("Une catégorie ne peut pas être son propre parent.".to_string()));
            }
            let _parent = CategoryRepository::find_by_id(db, pid, &model.tenant_id)
                .await?
                .ok_or_else(|| ApiError::BadRequest("Catégorie parente introuvable.".to_string()))?;
        }

        let is_active = payload.is_active.unwrap_or(model.is_active);
        let updated = CategoryRepository::update(
            db,
            id,
            &model.tenant_id,
            payload.name.as_deref().unwrap_or(&model.name),
            payload.description.or(model.description),
            payload.color.or(model.color),
            payload.icon.or(model.icon),
            payload.parent_id.or(model.parent_id),
            is_active,
        ).await?;

        Self::map_to_response(db, updated).await
    }

    pub async fn delete_category(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<CategoryResponse, ApiError> {
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
        use crate::models::category;

        let model = category::Entity::find()
            .filter(category::Column::Id.eq(id))
            .filter(category::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(|e| ApiError::Database(e))?
            .ok_or_else(|| ApiError::NotFound("Catégorie introuvable".to_string()))?;

        crate::utils::auth::require_tenant_access(db, caller_tenant_id, &model.tenant_id, caller_user_id, "delete").await?;

        // Check if it has children
        let children_count = category::Entity::find()
            .filter(category::Column::ParentId.eq(id))
            .filter(category::Column::DeletedAt.is_null())
            .count(db)
            .await
            .map_err(|e| ApiError::Database(e))?;

        if children_count > 0 {
            return Err(ApiError::BadRequest("Impossible de supprimer cette catégorie car elle contient des sous-catégories.".to_string()));
        }

        let deleted = CategoryRepository::soft_delete(db, id, &model.tenant_id).await?;
        Self::map_to_response(db, deleted).await
    }

    pub async fn map_to_response(
        db: &DatabaseConnection,
        model: crate::models::category::Model,
    ) -> Result<CategoryResponse, ApiError> {
        let parent_name = if let Some(ref pid) = model.parent_id {
            use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
            use crate::models::category;
            category::Entity::find()
                .filter(category::Column::Id.eq(pid))
                .filter(category::Column::DeletedAt.is_null())
                .one(db)
                .await
                .map_err(|e| ApiError::Database(e))?
                .map(|p| p.name)
        } else {
            None
        };

        Ok(CategoryResponse {
            id: model.id,
            tenant_id: model.tenant_id,
            name: model.name,
            description: model.description,
            color: model.color,
            icon: model.icon,
            parent_id: model.parent_id,
            parent_name,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }

    pub async fn map_to_response_list(
        db: &DatabaseConnection,
        models: Vec<crate::models::category::Model>,
    ) -> Result<Vec<CategoryResponse>, ApiError> {
        if models.is_empty() {
            return Ok(Vec::new());
        }

        let parent_ids: std::collections::HashSet<String> = models
            .iter()
            .filter_map(|m| m.parent_id.clone())
            .collect();

        let mut parent_names_map = std::collections::HashMap::new();
        if !parent_ids.is_empty() {
            use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
            use crate::models::category;
            let parents = category::Entity::find()
                .filter(category::Column::Id.is_in(parent_ids))
                .filter(category::Column::DeletedAt.is_null())
                .all(db)
                .await
                .map_err(|e| ApiError::Database(e))?;

            for p in parents {
                parent_names_map.insert(p.id, p.name);
            }
        }

        Ok(models
            .into_iter()
            .map(|m| {
                let parent_name = m.parent_id.as_ref().and_then(|pid| parent_names_map.get(pid).cloned());
                CategoryResponse {
                    id: m.id,
                    tenant_id: m.tenant_id,
                    name: m.name,
                    description: m.description,
                    color: m.color,
                    icon: m.icon,
                    parent_id: m.parent_id,
                    parent_name,
                    is_active: m.is_active,
                    created_at: m.created_at,
                    updated_at: m.updated_at,
                }
            })
            .collect())
    }
}
