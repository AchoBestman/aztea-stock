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
        ).await?;

        Ok(Self::map_to_response(created))
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

        Ok(Self::map_to_response(model))
    }

    pub async fn list_categories(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        target_tenant_id: Option<String>,
    ) -> Result<Vec<CategoryResponse>, ApiError> {
        let final_tenant_id = if let Some(t_id) = target_tenant_id {
            crate::utils::auth::require_tenant_access(db, caller_tenant_id, &t_id, caller_user_id, "read").await?;
            t_id
        } else {
            caller_tenant_id.to_string()
        };

        let categories = CategoryRepository::find_all(db, &final_tenant_id).await?;
        Ok(categories.into_iter().map(Self::map_to_response).collect())
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

        let updated = CategoryRepository::update(
            db,
            id,
            &model.tenant_id,
            payload.name.as_deref().unwrap_or(&model.name),
            payload.description.or(model.description),
            payload.color.or(model.color),
            payload.icon.or(model.icon),
            payload.parent_id.or(model.parent_id),
        ).await?;

        Ok(Self::map_to_response(updated))
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

        // Wait, products might be linked to this category! We will implement this check when products are added.
        
        let deleted = CategoryRepository::soft_delete(db, id, &model.tenant_id).await?;
        Ok(Self::map_to_response(deleted))
    }

    fn map_to_response(model: crate::models::category::Model) -> CategoryResponse {
        CategoryResponse {
            id: model.id,
            tenant_id: model.tenant_id,
            name: model.name,
            description: model.description,
            color: model.color,
            icon: model.icon,
            parent_id: model.parent_id,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
