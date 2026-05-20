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
        is_active: bool,
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
            is_active: Set(is_active),
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
        parent_id: Option<String>,
    ) -> Result<Vec<category::Model>, ApiError> {
        let mut query = category::Entity::find()
            .filter(category::Column::TenantId.eq(tenant_id))
            .filter(category::Column::DeletedAt.is_null());

        if let Some(pid) = parent_id {
            if pid.to_lowercase() == "null" {
                query = query.filter(category::Column::ParentId.is_null());
            } else {
                query = query.filter(category::Column::ParentId.eq(pid));
            }
        }

        query
            .order_by_asc(category::Column::Name)
            .all(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn find_all_paginated(
        db: &DatabaseConnection,
        tenant_id: &str,
        parent_id: Option<String>,
        is_active: Option<bool>,
        params: crate::utils::pagination::PaginationParams,
    ) -> Result<crate::utils::pagination::PaginatedResponse<category::Model>, ApiError> {
        let mut query = category::Entity::find()
            .filter(category::Column::TenantId.eq(tenant_id))
            .filter(category::Column::DeletedAt.is_null());

        if let Some(pid) = parent_id {
            if pid.to_lowercase() == "null" {
                query = query.filter(category::Column::ParentId.is_null());
            } else {
                query = query.filter(category::Column::ParentId.eq(pid));
            }
        }

        if let Some(active) = is_active {
            query = query.filter(category::Column::IsActive.eq(active));
        }

        if let Some(search) = params.search {
            query = query.filter(
                sea_orm::Condition::any()
                    .add(category::Column::Name.contains(&search))
                    .add(category::Column::Description.contains(&search))
            );
        }

        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20);
        let order_desc = params.order_type.as_deref().unwrap_or("desc") != "asc";

        let order_col = match params.order_by.as_deref().unwrap_or("name") {
            "created_at" => category::Column::CreatedAt,
            "updated_at" => category::Column::UpdatedAt,
            _ => category::Column::Name,
        };

        query = if order_desc {
            query.order_by_desc(order_col)
        } else {
            query.order_by_asc(order_col)
        };

        use sea_orm::PaginatorTrait;
        let paginator = query.paginate(db, per_page);
        let total = paginator.num_items().await.map_err(|e| ApiError::Database(e))?;
        let total_pages = paginator.num_pages().await.map_err(|e| ApiError::Database(e))?;

        let models = paginator.fetch_page(page - 1).await.map_err(|e| ApiError::Database(e))?;

        Ok(crate::utils::pagination::PaginatedResponse {
            data: models,
            total,
            page,
            per_page,
            total_pages,
        })
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
        is_active: bool,
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
        active_model.is_active = Set(is_active);
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
