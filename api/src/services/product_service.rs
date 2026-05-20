use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};
use validator::Validate;
use crate::{
    errors::ApiError,
    repositories::{product_repository::ProductRepository, category_repository::CategoryRepository},
    dtos::product_dto::{CreateProductPayload, UpdateProductPayload, ProductResponse},
    models::product,
};

pub struct ProductService;

impl ProductService {
    pub async fn create_product(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        target_tenant_id: Option<String>,
        payload: CreateProductPayload,
    ) -> Result<ProductResponse, ApiError> {
        payload.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

        let final_tenant_id = if let Some(t_id) = target_tenant_id {
            crate::utils::auth::require_tenant_access(db, caller_tenant_id, &t_id, caller_user_id, "create").await?;
            t_id
        } else {
            caller_tenant_id.to_string()
        };

        // If category_id is provided, verify it exists under this tenant
        if let Some(ref cid) = payload.category_id {
            let _cat = CategoryRepository::find_by_id(db, cid, &final_tenant_id)
                .await?
                .ok_or_else(|| ApiError::BadRequest("Catégorie spécifiée introuvable.".to_string()))?;
        }

        // If barcode is provided, check uniqueness within the tenant
        if let Some(ref bc) = payload.barcode {
            if !bc.trim().is_empty() {
                use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
                let existing = product::Entity::find()
                    .filter(product::Column::TenantId.eq(&final_tenant_id))
                    .filter(product::Column::Barcode.eq(bc))
                    .filter(product::Column::DeletedAt.is_null())
                    .one(db)
                    .await
                    .map_err(|e| ApiError::Database(e))?;
                if existing.is_some() {
                    return Err(ApiError::BadRequest("Un produit avec ce code-barres existe déjà pour ce tenant.".to_string()));
                }
            }
        }

        let id = uuid::Uuid::new_v4().to_string();
        let brand = payload.brand;
        let unit = payload.unit.unwrap_or_else(|| "unité".to_string());
        let purchase_price = payload.purchase_price.unwrap_or(0.0);
        let tax_rate = payload.tax_rate.unwrap_or(0.0);
        let is_active = payload.is_active.unwrap_or(true);
        let requires_prescription = payload.requires_prescription.unwrap_or(false);

        let created = ProductRepository::create(
            db,
            &id,
            &final_tenant_id,
            payload.category_id,
            payload.barcode,
            &payload.name,
            payload.description,
            brand,
            unit,
            purchase_price,
            payload.selling_price,
            tax_rate,
            payload.image_url,
            is_active,
            requires_prescription,
        ).await?;

        Self::map_to_response(db, created).await
    }

    pub async fn get_product(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<ProductResponse, ApiError> {
        let model = product::Entity::find()
            .filter(product::Column::Id.eq(id))
            .filter(product::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(|e| ApiError::Database(e))?
            .ok_or_else(|| ApiError::NotFound("Produit introuvable".to_string()))?;

        // Access guard
        crate::utils::auth::require_tenant_access(db, caller_tenant_id, &model.tenant_id, caller_user_id, "read").await?;

        Self::map_to_response(db, model).await
    }

    pub async fn list_products(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        target_tenant_id: Option<String>,
        category_id: Option<String>,
        is_active: Option<bool>,
        params: crate::utils::pagination::PaginationParams,
    ) -> Result<crate::utils::pagination::PaginatedResponse<ProductResponse>, ApiError> {
        let final_tenant_id = if let Some(t_id) = target_tenant_id {
            crate::utils::auth::require_tenant_access(db, caller_tenant_id, &t_id, caller_user_id, "read").await?;
            t_id
        } else {
            caller_tenant_id.to_string()
        };

        let paginated_models = ProductRepository::find_all_paginated(
            db,
            &final_tenant_id,
            category_id,
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

    pub async fn update_product(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
        payload: UpdateProductPayload,
    ) -> Result<ProductResponse, ApiError> {
        payload.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

        let model = product::Entity::find()
            .filter(product::Column::Id.eq(id))
            .filter(product::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(|e| ApiError::Database(e))?
            .ok_or_else(|| ApiError::NotFound("Produit introuvable".to_string()))?;

        // Guard
        crate::utils::auth::require_tenant_access(db, caller_tenant_id, &model.tenant_id, caller_user_id, "update").await?;

        // Verify category
        if let Some(Some(ref cid)) = payload.category_id {
            let _cat = CategoryRepository::find_by_id(db, cid, &model.tenant_id)
                .await?
                .ok_or_else(|| ApiError::BadRequest("Catégorie spécifiée introuvable.".to_string()))?;
        }

        // Verify barcode uniqueness if changed
        if let Some(Some(ref bc)) = payload.barcode {
            if !bc.trim().is_empty() && Some(bc.clone()) != model.barcode {
                use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
                let existing = product::Entity::find()
                    .filter(product::Column::TenantId.eq(&model.tenant_id))
                    .filter(product::Column::Barcode.eq(bc))
                    .filter(product::Column::DeletedAt.is_null())
                    .one(db)
                    .await
                    .map_err(|e| ApiError::Database(e))?;
                if existing.is_some() {
                    return Err(ApiError::BadRequest("Un produit avec ce code-barres existe déjà.".to_string()));
                }
            }
        }

        let final_name = payload.name.unwrap_or(model.name);
        let final_unit = payload.unit.unwrap_or(model.unit);
        let final_purchase_price = payload.purchase_price.unwrap_or(model.purchase_price);
        let final_selling_price = payload.selling_price.unwrap_or(model.selling_price);
        let final_tax_rate = payload.tax_rate.unwrap_or(model.tax_rate);
        let final_is_active = payload.is_active.unwrap_or(model.is_active);
        let final_requires_prescription = payload.requires_prescription.unwrap_or(model.requires_prescription);

        let updated = ProductRepository::update(
            db,
            id,
            &model.tenant_id,
            payload.category_id,
            payload.barcode,
            &final_name,
            payload.description,
            payload.brand,
            final_unit,
            final_purchase_price,
            final_selling_price,
            final_tax_rate,
            payload.image_url,
            final_is_active,
            final_requires_prescription,
        ).await?;

        Self::map_to_response(db, updated).await
    }

    pub async fn delete_product(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<ProductResponse, ApiError> {
        let model = product::Entity::find()
            .filter(product::Column::Id.eq(id))
            .filter(product::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(|e| ApiError::Database(e))?
            .ok_or_else(|| ApiError::NotFound("Produit introuvable".to_string()))?;

        // Guard
        crate::utils::auth::require_tenant_access(db, caller_tenant_id, &model.tenant_id, caller_user_id, "delete").await?;

        let deleted = ProductRepository::soft_delete(db, id, &model.tenant_id).await?;
        Self::map_to_response(db, deleted).await
    }

    pub async fn map_to_response(
        db: &DatabaseConnection,
        model: product::Model,
    ) -> Result<ProductResponse, ApiError> {
        let category_name = if let Some(ref cid) = model.category_id {
            CategoryRepository::find_by_id(db, cid, &model.tenant_id)
                .await?
                .map(|c| c.name)
        } else {
            None
        };

        Ok(ProductResponse {
            id: model.id,
            tenant_id: model.tenant_id,
            category_id: model.category_id,
            category_name,
            barcode: model.barcode,
            name: model.name,
            description: model.description,
            brand: model.brand,
            unit: model.unit,
            purchase_price: model.purchase_price,
            selling_price: model.selling_price,
            tax_rate: model.tax_rate,
            image_url: model.image_url,
            is_active: model.is_active,
            requires_prescription: model.requires_prescription,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }

    async fn map_to_response_list(
        db: &DatabaseConnection,
        models: Vec<product::Model>,
    ) -> Result<Vec<ProductResponse>, ApiError> {
        let mut list = Vec::with_capacity(models.len());
        for m in models {
            list.push(Self::map_to_response(db, m).await?);
        }
        Ok(list)
    }
}
