use crate::errors::ApiError;
use crate::models::{alerts, purchase_items, purchases, sale_items, sales, sync_log};
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};

pub struct GescomRepository;

impl GescomRepository {
    // --- Sales ---

    pub async fn create_sale(
        db: &impl sea_orm::ConnectionTrait,
        sale_model: sales::ActiveModel,
    ) -> Result<sales::Model, ApiError> {
        sale_model
            .insert(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn create_sale_item(
        db: &impl sea_orm::ConnectionTrait,
        item_model: sale_items::ActiveModel,
    ) -> Result<sale_items::Model, ApiError> {
        item_model
            .insert(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn find_sale_by_id(
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
    ) -> Result<Option<sales::Model>, ApiError> {
        sales::Entity::find()
            .filter(sales::Column::Id.eq(id))
            .filter(sales::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn find_sale_items(
        db: &impl sea_orm::ConnectionTrait,
        sale_id: &str,
    ) -> Result<Vec<sale_items::Model>, ApiError> {
        sale_items::Entity::find()
            .filter(sale_items::Column::SaleId.eq(sale_id))
            .all(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn find_sales_paginated(
        db: &impl sea_orm::ConnectionTrait,
        tenant_id: &str,
        customer_name: Option<String>,
        status: Option<String>,
        params: PaginationParams,
    ) -> Result<PaginatedResponse<sales::Model>, ApiError> {
        let mut query = sales::Entity::find().filter(sales::Column::TenantId.eq(tenant_id));

        if let Some(name) = customer_name {
            query = query.filter(
                sea_orm::Condition::any()
                    .add(sales::Column::CustomerName.contains(&name))
                    .add(sales::Column::CustomerPhone.contains(&name))
                    .add(sales::Column::ReceiptNumber.contains(&name)),
            );
        }

        if let Some(st) = status {
            query = query.filter(sales::Column::Status.eq(st));
        }

        if let Some(start) = params.start_date.as_ref() {
            query = query.filter(sales::Column::SoldAt.gte(start.clone()));
        }

        if let Some(end) = params.end_date.as_ref() {
            query = query.filter(sales::Column::SoldAt.lte(end.clone()));
        }

        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20);

        query = query.order_by_desc(sales::Column::SoldAt);

        let paginator = query.paginate(db, per_page);
        let total = paginator
            .num_items()
            .await
            .map_err(|e| ApiError::Database(e))?;
        let total_pages = paginator
            .num_pages()
            .await
            .map_err(|e| ApiError::Database(e))?;
        let models = paginator
            .fetch_page(page - 1)
            .await
            .map_err(|e| ApiError::Database(e))?;

        Ok(PaginatedResponse {
            data: models,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    pub async fn update_sale_status(
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
        new_status: &str,
    ) -> Result<sales::Model, ApiError> {
        let sale = Self::find_sale_by_id(db, id, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Vente introuvable".to_string()))?;

        let mut active_model: sales::ActiveModel = sale.into();
        active_model.status = Set(new_status.to_string());
        active_model
            .update(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    // --- Purchases ---

    pub async fn create_purchase(
        db: &impl sea_orm::ConnectionTrait,
        purchase_model: purchases::ActiveModel,
    ) -> Result<purchases::Model, ApiError> {
        purchase_model
            .insert(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn create_purchase_item(
        db: &impl sea_orm::ConnectionTrait,
        item_model: purchase_items::ActiveModel,
    ) -> Result<purchase_items::Model, ApiError> {
        item_model
            .insert(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn find_purchase_by_id(
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
    ) -> Result<Option<purchases::Model>, ApiError> {
        purchases::Entity::find()
            .filter(purchases::Column::Id.eq(id))
            .filter(purchases::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn find_purchase_items(
        db: &impl sea_orm::ConnectionTrait,
        purchase_id: &str,
    ) -> Result<Vec<purchase_items::Model>, ApiError> {
        purchase_items::Entity::find()
            .filter(purchase_items::Column::PurchaseId.eq(purchase_id))
            .all(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn find_purchases_paginated(
        db: &impl sea_orm::ConnectionTrait,
        tenant_id: &str,
        supplier_name: Option<String>,
        status: Option<String>,
        params: PaginationParams,
    ) -> Result<PaginatedResponse<purchases::Model>, ApiError> {
        let mut query = purchases::Entity::find().filter(purchases::Column::TenantId.eq(tenant_id));

        if let Some(name) = supplier_name {
            query = query.filter(purchases::Column::SupplierName.contains(&name));
        }

        if let Some(st) = status {
            query = query.filter(purchases::Column::Status.eq(st));
        }

        if let Some(start) = params.start_date.as_ref() {
            query = query.filter(purchases::Column::PurchasedAt.gte(start.clone()));
        }

        if let Some(end) = params.end_date.as_ref() {
            query = query.filter(purchases::Column::PurchasedAt.lte(end.clone()));
        }

        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20);

        query = query.order_by_desc(purchases::Column::PurchasedAt);

        let paginator = query.paginate(db, per_page);
        let total = paginator
            .num_items()
            .await
            .map_err(|e| ApiError::Database(e))?;
        let total_pages = paginator
            .num_pages()
            .await
            .map_err(|e| ApiError::Database(e))?;
        let models = paginator
            .fetch_page(page - 1)
            .await
            .map_err(|e| ApiError::Database(e))?;

        Ok(PaginatedResponse {
            data: models,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    pub async fn update_purchase_status(
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
        new_status: &str,
    ) -> Result<purchases::Model, ApiError> {
        let purchase = Self::find_purchase_by_id(db, id, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Achat introuvable".to_string()))?;

        let mut active_model: purchases::ActiveModel = purchase.into();
        active_model.status = Set(new_status.to_string());
        active_model
            .update(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    // --- Alerts ---

    pub async fn create_alert(
        db: &impl sea_orm::ConnectionTrait,
        tenant_id: &str,
        product_id: Option<String>,
        alert_type: &str,
        message: &str,
        threshold: Option<f64>,
        current_qty: Option<f64>,
    ) -> Result<alerts::Model, ApiError> {
        let id = uuid::Uuid::new_v4().to_string();
        let triggered_at: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
        let new_alert = alerts::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id.to_string()),
            product_id: Set(product_id),
            alert_type: Set(alert_type.to_string()),
            message: Set(message.to_string()),
            threshold: Set(threshold),
            current_qty: Set(current_qty),
            is_read: Set(false),
            is_resolved: Set(false),
            triggered_at: Set(triggered_at),
        };

        new_alert
            .insert(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn find_alert_by_id(
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
    ) -> Result<Option<alerts::Model>, ApiError> {
        alerts::Entity::find()
            .filter(alerts::Column::Id.eq(id))
            .filter(alerts::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn find_alerts_paginated(
        db: &impl sea_orm::ConnectionTrait,
        tenant_id: &str,
        is_read: Option<bool>,
        alert_type: Option<String>,
        params: PaginationParams,
    ) -> Result<PaginatedResponse<alerts::Model>, ApiError> {
        let mut query = alerts::Entity::find().filter(alerts::Column::TenantId.eq(tenant_id));

        if let Some(read) = is_read {
            query = query.filter(alerts::Column::IsRead.eq(read));
        }

        if let Some(t) = alert_type {
            query = query.filter(alerts::Column::AlertType.eq(t));
        }

        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20);

        query = query.order_by_desc(alerts::Column::TriggeredAt);

        let paginator = query.paginate(db, per_page);
        let total = paginator
            .num_items()
            .await
            .map_err(|e| ApiError::Database(e))?;
        let total_pages = paginator
            .num_pages()
            .await
            .map_err(|e| ApiError::Database(e))?;
        let models = paginator
            .fetch_page(page - 1)
            .await
            .map_err(|e| ApiError::Database(e))?;

        Ok(PaginatedResponse {
            data: models,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    pub async fn mark_alert_read(
        db: &impl sea_orm::ConnectionTrait,
        id: &str,
        tenant_id: &str,
    ) -> Result<alerts::Model, ApiError> {
        let alert = Self::find_alert_by_id(db, id, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Alerte introuvable".to_string()))?;

        let mut active_model: alerts::ActiveModel = alert.into();
        active_model.is_read = Set(true);
        active_model
            .update(db)
            .await
            .map_err(|e| ApiError::Database(e))
    }

    pub async fn mark_all_alerts_read(
        db: &impl sea_orm::ConnectionTrait,
        tenant_id: &str,
    ) -> Result<u64, ApiError> {
        let update_result = alerts::Entity::update_many()
            .col_expr(
                alerts::Column::IsRead,
                sea_orm::sea_query::Expr::value(true),
            )
            .filter(alerts::Column::TenantId.eq(tenant_id))
            .filter(alerts::Column::IsRead.eq(false))
            .exec(db)
            .await
            .map_err(|e| ApiError::Database(e))?;

        Ok(update_result.rows_affected)
    }

    // --- Sync Log ---

    pub async fn create_sync_log(
        db: &impl sea_orm::ConnectionTrait,
        tenant_id: &str,
        device_id: &str,
        sync_type: Option<String>,
        status: Option<String>,
        records_pushed: i32,
        records_pulled: i32,
        error_message: Option<String>,
    ) -> Result<sync_log::Model, ApiError> {
        let id = uuid::Uuid::new_v4().to_string();
        let started_at: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
        let finished_at: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
        let model = sync_log::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id.to_string()),
            device_id: Set(device_id.to_string()),
            sync_type: Set(sync_type),
            status: Set(status),
            records_pushed: Set(records_pushed),
            records_pulled: Set(records_pulled),
            error_message: Set(error_message),
            started_at: Set(started_at),
            finished_at: Set(Some(finished_at)),
        };

        model.insert(db).await.map_err(|e| ApiError::Database(e))
    }

    pub async fn find_sync_logs_paginated(
        db: &impl sea_orm::ConnectionTrait,
        tenant_id: &str,
        device_id: Option<String>,
        params: PaginationParams,
    ) -> Result<PaginatedResponse<sync_log::Model>, ApiError> {
        let mut query = sync_log::Entity::find().filter(sync_log::Column::TenantId.eq(tenant_id));

        if let Some(dev) = device_id {
            query = query.filter(sync_log::Column::DeviceId.eq(dev));
        }

        if let Some(start) = params.start_date.as_ref() {
            query = query.filter(sync_log::Column::StartedAt.gte(start.clone()));
        }

        if let Some(end) = params.end_date.as_ref() {
            query = query.filter(sync_log::Column::StartedAt.lte(end.clone()));
        }

        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20);

        query = query.order_by_desc(sync_log::Column::StartedAt);

        let paginator = query.paginate(db, per_page);
        let total = paginator
            .num_items()
            .await
            .map_err(|e| ApiError::Database(e))?;
        let total_pages = paginator
            .num_pages()
            .await
            .map_err(|e| ApiError::Database(e))?;
        let models = paginator
            .fetch_page(page - 1)
            .await
            .map_err(|e| ApiError::Database(e))?;

        Ok(PaginatedResponse {
            data: models,
            total,
            page,
            per_page,
            total_pages,
        })
    }
}
