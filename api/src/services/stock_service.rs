use sea_orm::DatabaseConnection;
use validator::Validate;
use crate::{
    errors::ApiError,
    repositories::{stock_repository::StockRepository, product_repository::ProductRepository, user_repository::UserRepository},
    dtos::stock_dto::{
        CreateStockItemPayload, UpdateStockItemPayload, StockItemResponse,
        CreateStockMovementPayload, StockMovementResponse
    },
    models::{stock_item, stock_movement},
};

pub struct StockService;

impl StockService {
    // --- Stock Items ---

    pub async fn create_stock_item(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        payload: CreateStockItemPayload,
    ) -> Result<StockItemResponse, ApiError> {
        payload.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

        // Verify product exists under this tenant
        let prod = ProductRepository::find_by_id(db, &payload.product_id, caller_tenant_id)
            .await?
            .ok_or_else(|| ApiError::BadRequest("Produit spécifié introuvable pour ce tenant.".to_string()))?;

        // Check if stock item already exists for this product
        if let Some(_) = StockRepository::find_stock_item_by_product_id(db, &payload.product_id, caller_tenant_id).await? {
            return Err(ApiError::BadRequest("Un article de stock existe déjà pour ce produit.".to_string()));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let quantity = payload.quantity.unwrap_or(0.0);
        let quantity_reserved = payload.quantity_reserved.unwrap_or(0.0);
        let low_stock_threshold = payload.low_stock_threshold.unwrap_or(5.0);

        let created = StockRepository::create_stock_item(
            db,
            &id,
            caller_tenant_id,
            &payload.product_id,
            quantity,
            quantity_reserved,
            low_stock_threshold,
            payload.unit_location,
            payload.batch_number,
            payload.expiry_date,
        ).await?;

        // If initial quantity is greater than 0, create an initial stock movement
        if quantity > 0.0 {
            let movement_id = uuid::Uuid::new_v4().to_string();
            StockRepository::create_stock_movement(
                db,
                &movement_id,
                caller_tenant_id,
                &payload.product_id,
                Some(caller_user_id.to_string()),
                "initial",
                0.0,
                quantity,
                quantity,
                None,
                Some("Stock initial lors de la création de la fiche".to_string()),
            ).await?;
        }

        Self::map_item_to_response(db, created, &prod.name).await
    }

    pub async fn get_stock_item(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<StockItemResponse, ApiError> {
        let model = StockRepository::find_stock_item_by_id(db, id, caller_tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Article de stock introuvable".to_string()))?;

        crate::utils::auth::require_tenant_access(db, caller_tenant_id, &model.tenant_id, caller_user_id, "read").await?;

        let prod = ProductRepository::find_by_id(db, &model.product_id, caller_tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Produit associé introuvable".to_string()))?;

        Self::map_item_to_response(db, model, &prod.name).await
    }

    pub async fn list_stock_items(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        category_id: Option<String>,
        is_low_stock: Option<bool>,
        params: crate::utils::pagination::PaginationParams,
    ) -> Result<crate::utils::pagination::PaginatedResponse<StockItemResponse>, ApiError> {
        let final_tenant_id = if let Some(ref t_id) = params.tenant_id {
            crate::utils::auth::require_tenant_access(db, caller_tenant_id, t_id, caller_user_id, "read").await?;
            t_id.clone()
        } else {
            caller_tenant_id.to_string()
        };

        let paginated = StockRepository::find_stock_items_paginated(
            db,
            &final_tenant_id,
            category_id,
            is_low_stock,
            params,
        ).await?;

        let mut data = Vec::with_capacity(paginated.data.len());
        for item in paginated.data {
            let prod_name = ProductRepository::find_by_id(db, &item.product_id, &final_tenant_id)
                .await?
                .map(|p| p.name)
                .unwrap_or_default();
            data.push(Self::map_item_to_response(db, item, &prod_name).await?);
        }

        Ok(crate::utils::pagination::PaginatedResponse {
            data,
            total: paginated.total,
            page: paginated.page,
            per_page: paginated.per_page,
            total_pages: paginated.total_pages,
        })
    }

    pub async fn update_stock_item(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
        payload: UpdateStockItemPayload,
    ) -> Result<StockItemResponse, ApiError> {
        payload.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

        let item = StockRepository::find_stock_item_by_id(db, id, caller_tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Article de stock introuvable".to_string()))?;

        crate::utils::auth::require_tenant_access(db, caller_tenant_id, &item.tenant_id, caller_user_id, "update").await?;

        let final_qty = payload.quantity.unwrap_or(item.quantity);
        let final_reserved = payload.quantity_reserved.unwrap_or(item.quantity_reserved);
        let final_threshold = payload.low_stock_threshold.unwrap_or(item.low_stock_threshold);

        // If the quantity was changed manually, register an adjustment movement
        if final_qty != item.quantity {
            let movement_id = uuid::Uuid::new_v4().to_string();
            StockRepository::create_stock_movement(
                db,
                &movement_id,
                caller_tenant_id,
                &item.product_id,
                Some(caller_user_id.to_string()),
                "adjustment",
                item.quantity,
                final_qty - item.quantity,
                final_qty,
                None,
                Some("Ajustement manuel de quantité".to_string()),
            ).await?;
        }

        let updated = StockRepository::update_stock_item(
            db,
            id,
            caller_tenant_id,
            final_qty,
            final_reserved,
            final_threshold,
            payload.unit_location,
            payload.batch_number,
            payload.expiry_date,
        ).await?;

        let prod = ProductRepository::find_by_id(db, &updated.product_id, caller_tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Produit associé introuvable".to_string()))?;

        Self::map_item_to_response(db, updated, &prod.name).await
    }

    pub async fn delete_stock_item(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<(), ApiError> {
        let item = StockRepository::find_stock_item_by_id(db, id, caller_tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Article de stock introuvable".to_string()))?;

        crate::utils::auth::require_tenant_access(db, caller_tenant_id, &item.tenant_id, caller_user_id, "delete").await?;

        StockRepository::delete_stock_item(db, id, caller_tenant_id).await
    }

    // --- Stock Movements ---

    pub async fn create_stock_movement(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        payload: CreateStockMovementPayload,
    ) -> Result<StockMovementResponse, ApiError> {
        payload.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

        // Verify product exists
        let prod = ProductRepository::find_by_id(db, &payload.product_id, caller_tenant_id)
            .await?
            .ok_or_else(|| ApiError::BadRequest("Produit spécifié introuvable pour ce tenant.".to_string()))?;

        // Try to find existing stock item, or create one if it doesn't exist yet
        let item = match StockRepository::find_stock_item_by_product_id(db, &payload.product_id, caller_tenant_id).await? {
            Some(i) => i,
            None => {
                let id = uuid::Uuid::new_v4().to_string();
                StockRepository::create_stock_item(
                    db,
                    &id,
                    caller_tenant_id,
                    &payload.product_id,
                    0.0,
                    0.0,
                    5.0,
                    None,
                    None,
                    None,
                ).await?
            }
        };

        let qty_before = item.quantity;
        let qty_change = payload.quantity_change;
        let qty_after = qty_before + qty_change;

        if qty_after < 0.0 {
            return Err(ApiError::BadRequest("Le stock ne peut pas devenir négatif.".to_string()));
        }

        // 1. Update stock item quantity
        StockRepository::update_stock_item(
            db,
            &item.id,
            caller_tenant_id,
            qty_after,
            item.quantity_reserved,
            item.low_stock_threshold,
            None,
            None,
            None,
        ).await?;

        // 2. Create the movement record
        let movement_id = uuid::Uuid::new_v4().to_string();
        let movement = StockRepository::create_stock_movement(
            db,
            &movement_id,
            caller_tenant_id,
            &payload.product_id,
            Some(caller_user_id.to_string()),
            &payload.movement_type,
            qty_before,
            qty_change,
            qty_after,
            payload.reference_id,
            payload.note,
        ).await?;

        // 3. Map to response
        let caller_name = UserRepository::find_by_id(db, caller_user_id, caller_tenant_id)
            .await?
            .map(|u| u.name);

        Self::map_movement_to_response(db, movement, &prod.name, caller_name).await
    }

    pub async fn list_stock_movements(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        product_id: Option<String>,
        movement_type: Option<String>,
        params: crate::utils::pagination::PaginationParams,
    ) -> Result<crate::utils::pagination::PaginatedResponse<StockMovementResponse>, ApiError> {
        let final_tenant_id = if let Some(ref t_id) = params.tenant_id {
            crate::utils::auth::require_tenant_access(db, caller_tenant_id, t_id, caller_user_id, "read").await?;
            t_id.clone()
        } else {
            caller_tenant_id.to_string()
        };

        let paginated = StockRepository::find_stock_movements_paginated(
            db,
            &final_tenant_id,
            product_id,
            movement_type,
            params,
        ).await?;

        let mut data = Vec::with_capacity(paginated.data.len());
        for m in paginated.data {
            let prod_name = ProductRepository::find_by_id(db, &m.product_id, &final_tenant_id)
                .await?
                .map(|p| p.name)
                .unwrap_or_default();
            let u_name = if let Some(ref uid) = m.user_id {
                UserRepository::find_by_id(db, uid, &final_tenant_id).await?.map(|u| u.name)
            } else {
                None
            };
            data.push(Self::map_movement_to_response(db, m, &prod_name, u_name).await?);
        }

        Ok(crate::utils::pagination::PaginatedResponse {
            data,
            total: paginated.total,
            page: paginated.page,
            per_page: paginated.per_page,
            total_pages: paginated.total_pages,
        })
    }

    // --- Helpers ---

    async fn map_item_to_response(
        _db: &DatabaseConnection,
        model: stock_item::Model,
        product_name: &str,
    ) -> Result<StockItemResponse, ApiError> {
        Ok(StockItemResponse {
            id: model.id,
            tenant_id: model.tenant_id,
            product_id: model.product_id,
            product_name: product_name.to_string(),
            quantity: model.quantity,
            quantity_reserved: model.quantity_reserved,
            low_stock_threshold: model.low_stock_threshold,
            unit_location: model.unit_location,
            batch_number: model.batch_number,
            expiry_date: model.expiry_date,
            updated_at: model.updated_at,
        })
    }

    async fn map_movement_to_response(
        _db: &DatabaseConnection,
        model: stock_movement::Model,
        product_name: &str,
        user_name: Option<String>,
    ) -> Result<StockMovementResponse, ApiError> {
        Ok(StockMovementResponse {
            id: model.id,
            tenant_id: model.tenant_id,
            product_id: model.product_id,
            product_name: product_name.to_string(),
            user_id: model.user_id,
            user_name,
            movement_type: model.movement_type,
            quantity_before: model.quantity_before,
            quantity_change: model.quantity_change,
            quantity_after: model.quantity_after,
            reference_id: model.reference_id,
            note: model.note,
            occurred_at: model.occurred_at,
        })
    }
}
