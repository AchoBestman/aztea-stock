use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, QueryOrder};
use crate::models::category;
use crate::errors::ApiError;

pub struct CategoryRepository;

impl CategoryRepository {
    pub async fn create(
        db: &DatabaseConnection,
        id: &str,
        tenant_id: &str,
        name: &str,
        description: Option<String>,
        color: Option<String>,
        icon: Option<String>,
        parent_id: Option<String>,
    ) -> Result<category::Model, ApiError> {
        let now = chrono::Utc::now().to_rfc3339();
        let new_category = category::ActiveModel {
            id: Set(id.to_string()),
            tenant_id: Set(tenant_id.to_string()),
            name: Set(name.to_string()),
            description: Set(description),
            color: Set(color),
            icon: Set(icon),
            parent_id: Set(parent_id),
            created_at: Set(now.clone()),
            updated_at: Set(now),
            deleted_at: Set(None),
        };

        new_category.insert(db).await.map_err(|e| ApiError::Database(e))
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: &str,
        tenant_id: &str,
    ) -> Result<Option<category::Model>, ApiError> {
        category::Entity::find()
            .filter(category::Column::Id.eq(id))
            .filter(category::Column::TenantId.eq(tenant_id))
            .filter(category::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn find_all(
        db: &DatabaseConnection,
        tenant_id: &str,
    ) -> Result<Vec<category::Model>, ApiError> {
        category::Entity::find()
            .filter(category::Column::TenantId.eq(tenant_id))
            .filter(category::Column::DeletedAt.is_null())
            .order_by_asc(category::Column::Name)
            .all(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn update(
        db: &DatabaseConnection,
        id: &str,
        tenant_id: &str,
        name: &str,
        description: Option<String>,
        color: Option<String>,
        icon: Option<String>,
        parent_id: Option<String>,
    ) -> Result<category::Model, ApiError> {
        let model = Self::find_by_id(db, id, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Catégorie introuvable".to_string()))?;

        let mut active_model: category::ActiveModel = model.into();
        active_model.name = Set(name.to_string());
        active_model.description = Set(description);
        active_model.color = Set(color);
        active_model.icon = Set(icon);
        active_model.parent_id = Set(parent_id);
        active_model.updated_at = Set(chrono::Utc::now().to_rfc3339());

        active_model.update(db).await.map_err(|e| ApiError::Database(e))
    }

    pub async fn soft_delete(
        db: &DatabaseConnection,
        id: &str,
        tenant_id: &str,
    ) -> Result<category::Model, ApiError> {
        let model = Self::find_by_id(db, id, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Catégorie introuvable".to_string()))?;

        let mut active_model: category::ActiveModel = model.into();
        active_model.deleted_at = Set(Some(chrono::Utc::now().to_rfc3339()));

        active_model.update(db).await.map_err(|e| ApiError::Database(e))
    }
}
