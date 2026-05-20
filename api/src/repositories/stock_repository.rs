use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, QueryOrder, QuerySelect, RelationTrait};
use crate::models::{stock_item, stock_movement, product};
use crate::errors::ApiError;

pub struct StockRepository;

impl StockRepository {
    // --- Stock Items ---
    
    pub async fn create_stock_item(
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
        product_id: &str,
        quantity: f64,
        quantity_reserved: f64,
        low_stock_threshold: f64,
        unit_location: Option<String>,
        batch_number: Option<String>,
        expiry_date: Option<String>,
    ) -> Result<stock_item::Model, ApiError> {
        let now = chrono::Utc::now().to_rfc3339();
        let new_item = stock_item::ActiveModel {
            id: Set(id.to_string()),
            tenant_id: Set(tenant_id.to_string()),
            product_id: Set(product_id.to_string()),
            quantity: Set(quantity),
            quantity_reserved: Set(quantity_reserved),
            low_stock_threshold: Set(low_stock_threshold),
            unit_location: Set(unit_location),
            batch_number: Set(batch_number),
            expiry_date: Set(expiry_date),
            updated_at: Set(now),
        };

        new_item.insert(db).await.map_err(|e| ApiError::Database(e))
    }

    pub async fn find_stock_item_by_id(
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
    ) -> Result<Option<stock_item::Model>, ApiError> {
        stock_item::Entity::find()
            .filter(stock_item::Column::Id.eq(id))
            .filter(stock_item::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn find_stock_item_by_product_id(
        db: &impl sea_orm::ConnectionTrait,
        product_id: &str,
        tenant_id: &str,
    ) -> Result<Option<stock_item::Model>, ApiError> {
        stock_item::Entity::find()
            .filter(stock_item::Column::ProductId.eq(product_id))
            .filter(stock_item::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn find_stock_items_paginated(
        db: &impl sea_orm::ConnectionTrait,
        tenant_id: &str,
        category_id: Option<String>,
        is_low_stock: Option<bool>,
        params: crate::utils::pagination::PaginationParams,
    ) -> Result<crate::utils::pagination::PaginatedResponse<stock_item::Model>, ApiError> {
        let mut query = stock_item::Entity::find()
            .filter(stock_item::Column::TenantId.eq(tenant_id));

        // Join products for advanced filters
        if category_id.is_some() || params.search.is_some() {
            query = query.join(
                sea_orm::JoinType::InnerJoin,
                stock_item::Relation::Product.def()
            );
        }

        if let Some(cid) = category_id {
            query = query.filter(product::Column::CategoryId.eq(cid));
        }

        if let Some(search) = params.search {
            query = query.filter(
                sea_orm::Condition::any()
                    .add(product::Column::Name.contains(&search))
                    .add(product::Column::Barcode.contains(&search))
                    .add(stock_item::Column::BatchNumber.contains(&search))
            );
        }

        if let Some(low) = is_low_stock {
            if low {
                query = query.filter(
                    sea_orm::sea_query::Expr::col(stock_item::Column::Quantity)
                        .lte(sea_orm::sea_query::Expr::col(stock_item::Column::LowStockThreshold))
                );
            }
        }

        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20);
        let order_desc = params.order_type.as_deref().unwrap_or("desc") != "asc";

        let order_col = match params.order_by.as_deref().unwrap_or("updated_at") {
            "quantity" => stock_item::Column::Quantity,
            "expiry_date" => stock_item::Column::ExpiryDate,
            _ => stock_item::Column::UpdatedAt,
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

    pub async fn update_stock_item(
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
        quantity: f64,
        quantity_reserved: f64,
        low_stock_threshold: f64,
        unit_location: Option<Option<String>>,
        batch_number: Option<Option<String>>,
        expiry_date: Option<Option<String>>,
    ) -> Result<stock_item::Model, ApiError> {
        let model = Self::find_stock_item_by_id(db, id, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Article de stock introuvable".to_string()))?;

        let mut active_model: stock_item::ActiveModel = model.into();
        active_model.quantity = Set(quantity);
        active_model.quantity_reserved = Set(quantity_reserved);
        active_model.low_stock_threshold = Set(low_stock_threshold);
        if let Some(loc) = unit_location {
            active_model.unit_location = Set(loc);
        }
        if let Some(bn) = batch_number {
            active_model.batch_number = Set(bn);
        }
        if let Some(exp) = expiry_date {
            active_model.expiry_date = Set(exp);
        }
        active_model.updated_at = Set(chrono::Utc::now().to_rfc3339());

        active_model.update(db).await.map_err(|e| ApiError::Database(e))
    }

    pub async fn delete_stock_item(
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
    ) -> Result<(), ApiError> {
        let model = Self::find_stock_item_by_id(db, id, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Article de stock introuvable".to_string()))?;

        let active_model: stock_item::ActiveModel = model.into();
        active_model.delete(db).await.map_err(|e| ApiError::Database(e))?;
        Ok(())
    }

    // --- Stock Movements ---

    pub async fn create_stock_movement(
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
        product_id: &str,
        user_id: Option<String>,
        movement_type: &str,
        quantity_before: f64,
        quantity_change: f64,
        quantity_after: f64,
        reference_id: Option<String>,
        note: Option<String>,
    ) -> Result<stock_movement::Model, ApiError> {
        let occurred_at = chrono::Utc::now().to_rfc3339();
        let new_movement = stock_movement::ActiveModel {
            id: Set(id.to_string()),
            tenant_id: Set(tenant_id.to_string()),
            product_id: Set(product_id.to_string()),
            user_id: Set(user_id),
            movement_type: Set(movement_type.to_string()),
            quantity_before: Set(quantity_before),
            quantity_change: Set(quantity_change),
            quantity_after: Set(quantity_after),
            reference_id: Set(reference_id),
            note: Set(note),
            occurred_at: Set(occurred_at),
        };

        new_movement.insert(db).await.map_err(|e| ApiError::Database(e))
    }

    pub async fn find_stock_movement_by_id(
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
    ) -> Result<Option<stock_movement::Model>, ApiError> {
        stock_movement::Entity::find()
            .filter(stock_movement::Column::Id.eq(id))
            .filter(stock_movement::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn find_stock_movements_paginated(
        db: &impl sea_orm::ConnectionTrait,
        tenant_id: &str,
        product_id: Option<String>,
        movement_type: Option<String>,
        params: crate::utils::pagination::PaginationParams,
    ) -> Result<crate::utils::pagination::PaginatedResponse<stock_movement::Model>, ApiError> {
        let mut query = stock_movement::Entity::find()
            .filter(stock_movement::Column::TenantId.eq(tenant_id));

        if let Some(pid) = product_id {
            query = query.filter(stock_movement::Column::ProductId.eq(pid));
        }

        if let Some(m_type) = movement_type {
            query = query.filter(stock_movement::Column::MovementType.eq(m_type));
        }

        if let Some(start) = params.start_date.as_ref() {
            query = query.filter(stock_movement::Column::OccurredAt.gte(start.clone()));
        }

        if let Some(end) = params.end_date.as_ref() {
            query = query.filter(stock_movement::Column::OccurredAt.lte(end.clone()));
        }

        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20);

        // Movements are always chronologically ordered by default
        query = query.order_by_desc(stock_movement::Column::OccurredAt);

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
}
