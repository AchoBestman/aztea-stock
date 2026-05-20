use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, QueryOrder};
use crate::models::product;
use crate::errors::ApiError;

pub struct ProductRepository;

impl ProductRepository {
    pub async fn create(
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
        category_id: Option<String>,
        barcode: Option<String>,
        name: &str,
        description: Option<String>,
        brand: Option<String>,
        unit: String,
        purchase_price: f64,
        selling_price: f64,
        tax_rate: f64,
        image_url: Option<String>,
        is_active: bool,
        requires_prescription: bool,
    ) -> Result<product::Model, ApiError> {
        let now = chrono::Utc::now().to_rfc3339();
        let new_product = product::ActiveModel {
            id: Set(id.to_string()),
            tenant_id: Set(tenant_id.to_string()),
            category_id: Set(category_id),
            barcode: Set(barcode),
            name: Set(name.to_string()),
            description: Set(description),
            brand: Set(brand),
            unit: Set(unit),
            purchase_price: Set(purchase_price),
            selling_price: Set(selling_price),
            tax_rate: Set(tax_rate),
            image_url: Set(image_url),
            is_active: Set(is_active),
            requires_prescription: Set(requires_prescription),
            created_at: Set(now.clone()),
            updated_at: Set(now),
            deleted_at: Set(None),
        };

        new_product.insert(db).await.map_err(|e| ApiError::Database(e))
    }

    pub async fn find_by_id(
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
    ) -> Result<Option<product::Model>, ApiError> {
        product::Entity::find()
            .filter(product::Column::Id.eq(id))
            .filter(product::Column::TenantId.eq(tenant_id))
            .filter(product::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn find_all_paginated(
        db: &impl sea_orm::ConnectionTrait,
        tenant_id: &str,
        category_id: Option<String>,
        is_active: Option<bool>,
        params: crate::utils::pagination::PaginationParams,
    ) -> Result<crate::utils::pagination::PaginatedResponse<product::Model>, ApiError> {
        let mut query = product::Entity::find()
            .filter(product::Column::TenantId.eq(tenant_id))
            .filter(product::Column::DeletedAt.is_null());

        if let Some(cid) = category_id {
            query = query.filter(product::Column::CategoryId.eq(cid));
        }

        if let Some(active) = is_active {
            query = query.filter(product::Column::IsActive.eq(active));
        }

        if let Some(search) = params.search {
            query = query.filter(
                sea_orm::Condition::any()
                    .add(product::Column::Name.contains(&search))
                    .add(product::Column::Barcode.contains(&search))
                    .add(product::Column::Description.contains(&search))
                    .add(product::Column::Brand.contains(&search))
            );
        }

        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20);
        let order_desc = params.order_type.as_deref().unwrap_or("desc") != "asc";

        let order_col = match params.order_by.as_deref().unwrap_or("name") {
            "created_at" => product::Column::CreatedAt,
            "updated_at" => product::Column::UpdatedAt,
            "selling_price" => product::Column::SellingPrice,
            _ => product::Column::Name,
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
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
        category_id: Option<Option<String>>,
        barcode: Option<Option<String>>,
        name: &str,
        description: Option<Option<String>>,
        brand: Option<Option<String>>,
        unit: String,
        purchase_price: f64,
        selling_price: f64,
        tax_rate: f64,
        image_url: Option<Option<String>>,
        is_active: bool,
        requires_prescription: bool,
    ) -> Result<product::Model, ApiError> {
        let model = Self::find_by_id(db, id, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Produit introuvable".to_string()))?;

        let mut active_model: product::ActiveModel = model.into();
        
        if let Some(cid) = category_id {
            active_model.category_id = Set(cid);
        }
        if let Some(bc) = barcode {
            active_model.barcode = Set(bc);
        }
        active_model.name = Set(name.to_string());
        if let Some(desc) = description {
            active_model.description = Set(desc);
        }
        if let Some(b) = brand {
            active_model.brand = Set(b);
        }
        active_model.unit = Set(unit);
        active_model.purchase_price = Set(purchase_price);
        active_model.selling_price = Set(selling_price);
        active_model.tax_rate = Set(tax_rate);
        if let Some(img) = image_url {
            active_model.image_url = Set(img);
        }
        active_model.is_active = Set(is_active);
        active_model.requires_prescription = Set(requires_prescription);
        active_model.updated_at = Set(chrono::Utc::now().to_rfc3339());

        active_model.update(db).await.map_err(|e| ApiError::Database(e))
    }

    pub async fn soft_delete(
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
    ) -> Result<product::Model, ApiError> {
        let model = Self::find_by_id(db, id, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Produit introuvable".to_string()))?;

        let mut active_model: product::ActiveModel = model.into();
        active_model.deleted_at = Set(Some(chrono::Utc::now().to_rfc3339()));

        active_model.update(db).await.map_err(|e| ApiError::Database(e))
    }
}
