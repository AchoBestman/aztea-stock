use axum::{
    Json, Extension, extract::{Path, State, Query}
};
use std::sync::Arc;
use serde::Deserialize;
use utoipa::IntoParams;
use crate::{
    AppState,
    errors::ApiError,
    middleware::auth::Claims,
    dtos::gescom_dto::{
        CreateSalePayload, SaleResponse, PaginatedSaleResponse, RefundSalePayload, ReceiptPrintResponse,
        CreatePurchasePayload, PurchaseResponse, PaginatedPurchaseResponse,
        AlertResponse, PaginatedAlertResponse,
        CreateSyncLogPayload, SyncLogResponse, PaginatedSyncLogResponse,
    },
    services::gescom_service::GescomService,
    utils::auth::{require_permission, require_tenant_access},
};

// --- Queries ---

#[derive(Deserialize, IntoParams)]
pub struct ListSalesQuery {
    pub tenant_id: Option<String>,
    pub customer_name: Option<String>,
    pub status: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Deserialize, IntoParams)]
pub struct ListPurchasesQuery {
    pub tenant_id: Option<String>,
    pub supplier_name: Option<String>,
    pub status: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Deserialize, IntoParams)]
pub struct ListAlertsQuery {
    pub tenant_id: Option<String>,
    pub is_read: Option<bool>,
    pub alert_type: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Deserialize, IntoParams)]
pub struct ListSyncLogsQuery {
    pub tenant_id: Option<String>,
    pub device_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

// --- Sales Endpoints ---

#[utoipa::path(
    post,
    path = "/api/v1/sales",
    request_body = CreateSalePayload,
    responses(
        (status = 201, description = "Vente enregistrée avec succès.", body = SaleResponse),
        (status = 400, description = "Données invalides ou stock insuffisant."),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Ventes"
)]
pub async fn create_sale(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateSalePayload>,
) -> Result<Json<SaleResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_create_sale").await?;

    let target_tenant_id = payload.tenant_id.as_deref().unwrap_or(&claims.tenant_id).to_string();
    require_tenant_access(db, &claims.tenant_id, &target_tenant_id, &claims.sub, "create").await?;

    let sale = GescomService::create_sale(db, &claims.sub, &target_tenant_id, payload).await?;
    Ok(Json(sale))
}

#[utoipa::path(
    get,
    path = "/api/v1/sales",
    params(ListSalesQuery),
    responses(
        (status = 200, description = "Liste paginée des ventes.", body = PaginatedSaleResponse),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Ventes"
)]
pub async fn list_sales(
    Query(query): Query<ListSalesQuery>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<PaginatedSaleResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_sale").await?;

    let params = crate::utils::pagination::PaginationParams {
        page: query.page,
        per_page: query.per_page,
        search: None,
        start_date: query.start_date,
        end_date: query.end_date,
        tenant_id: query.tenant_id,
        order_by: None,
        order_type: None,
    };

    let response = GescomService::list_sales(
        db,
        &claims.sub,
        &claims.tenant_id,
        query.customer_name,
        query.status,
        params,
    ).await?;

    Ok(Json(PaginatedSaleResponse {
        data: response.data,
        total: response.total,
        page: response.page,
        per_page: response.per_page,
        total_pages: response.total_pages,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/sales/{id}",
    params(
        ("id" = String, Path, description = "ID UUID de la vente")
    ),
    responses(
        (status = 200, description = "Détail de la vente.", body = SaleResponse),
        (status = 401, description = "Authentification requise."),
        (status = 404, description = "Vente introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Ventes"
)]
pub async fn get_sale(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<SaleResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_sale").await?;

    let sale = GescomService::get_sale(db, &id, &claims.sub, &claims.tenant_id).await?;
    Ok(Json(sale))
}

#[utoipa::path(
    post,
    path = "/api/v1/sales/{id}/void",
    params(
        ("id" = String, Path, description = "ID UUID de la vente")
    ),
    responses(
        (status = 200, description = "Vente annulée avec succès.", body = SaleResponse),
        (status = 400, description = "Impossible d'annuler cette vente."),
        (status = 401, description = "Authentification requise."),
        (status = 404, description = "Vente introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Ventes"
)]
pub async fn void_sale(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<SaleResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_update_sale").await?;

    let sale = GescomService::void_sale(db, &id, &claims.sub, &claims.tenant_id).await?;
    Ok(Json(sale))
}

#[utoipa::path(
    post,
    path = "/api/v1/sales/{id}/refund",
    params(
        ("id" = String, Path, description = "ID UUID de la vente")
    ),
    request_body = RefundSalePayload,
    responses(
        (status = 200, description = "Remboursement enregistré avec succès.", body = SaleResponse),
        (status = 400, description = "Impossible de rembourser cette vente."),
        (status = 401, description = "Authentification requise."),
        (status = 404, description = "Vente introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Ventes"
)]
pub async fn refund_sale(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefundSalePayload>,
) -> Result<Json<SaleResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_update_sale").await?;

    let sale = GescomService::refund_sale(db, &id, &claims.sub, &claims.tenant_id, payload).await?;
    Ok(Json(sale))
}

#[utoipa::path(
    get,
    path = "/api/v1/sales/{id}/receipt",
    params(
        ("id" = String, Path, description = "ID UUID de la vente")
    ),
    responses(
        (status = 200, description = "Données de reçu générées.", body = ReceiptPrintResponse),
        (status = 401, description = "Authentification requise."),
        (status = 404, description = "Vente introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Ventes"
)]
pub async fn get_sale_receipt(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ReceiptPrintResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_sale").await?;

    let receipt = GescomService::get_sale_receipt(db, &id, &claims.sub, &claims.tenant_id).await?;
    Ok(Json(receipt))
}

// --- Purchases Endpoints ---

#[utoipa::path(
    post,
    path = "/api/v1/purchases",
    request_body = CreatePurchasePayload,
    responses(
        (status = 201, description = "Achat enregistré avec succès.", body = PurchaseResponse),
        (status = 400, description = "Données invalides."),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Achats"
)]
pub async fn create_purchase(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePurchasePayload>,
) -> Result<Json<PurchaseResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_create_purchase").await?;

    let target_tenant_id = payload.tenant_id.as_deref().unwrap_or(&claims.tenant_id).to_string();
    require_tenant_access(db, &claims.tenant_id, &target_tenant_id, &claims.sub, "create").await?;

    let purchase = GescomService::create_purchase(db, &claims.sub, &target_tenant_id, payload).await?;
    Ok(Json(purchase))
}

#[utoipa::path(
    get,
    path = "/api/v1/purchases",
    params(ListPurchasesQuery),
    responses(
        (status = 200, description = "Liste paginée des achats.", body = PaginatedPurchaseResponse),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Achats"
)]
pub async fn list_purchases(
    Query(query): Query<ListPurchasesQuery>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<PaginatedPurchaseResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_purchase").await?;

    let params = crate::utils::pagination::PaginationParams {
        page: query.page,
        per_page: query.per_page,
        search: None,
        start_date: query.start_date,
        end_date: query.end_date,
        tenant_id: query.tenant_id,
        order_by: None,
        order_type: None,
    };

    let response = GescomService::list_purchases(
        db,
        &claims.sub,
        &claims.tenant_id,
        query.supplier_name,
        query.status,
        params,
    ).await?;

    Ok(Json(PaginatedPurchaseResponse {
        data: response.data,
        total: response.total,
        page: response.page,
        per_page: response.per_page,
        total_pages: response.total_pages,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/purchases/{id}",
    params(
        ("id" = String, Path, description = "ID UUID de l'achat")
    ),
    responses(
        (status = 200, description = "Détail de l'achat.", body = PurchaseResponse),
        (status = 401, description = "Authentification requise."),
        (status = 404, description = "Achat introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Achats"
)]
pub async fn get_purchase(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<PurchaseResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_purchase").await?;

    let purchase = GescomService::get_purchase(db, &id, &claims.sub, &claims.tenant_id).await?;
    Ok(Json(purchase))
}

#[utoipa::path(
    post,
    path = "/api/v1/purchases/{id}/cancel",
    params(
        ("id" = String, Path, description = "ID UUID de l'achat")
    ),
    responses(
        (status = 200, description = "Achat annulé avec succès.", body = PurchaseResponse),
        (status = 400, description = "Impossible d'annuler cet achat (deviendrait négatif)."),
        (status = 401, description = "Authentification requise."),
        (status = 404, description = "Achat introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Achats"
)]
pub async fn cancel_purchase(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<PurchaseResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_update_purchase").await?;

    let purchase = GescomService::cancel_purchase(db, &id, &claims.sub, &claims.tenant_id).await?;
    Ok(Json(purchase))
}

// --- Alerts Endpoints ---

#[utoipa::path(
    get,
    path = "/api/v1/alerts",
    params(ListAlertsQuery),
    responses(
        (status = 200, description = "Liste paginée des alertes.", body = PaginatedAlertResponse),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Alertes"
)]
pub async fn list_alerts(
    Query(query): Query<ListAlertsQuery>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<PaginatedAlertResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_alert").await?;

    let params = crate::utils::pagination::PaginationParams {
        page: query.page,
        per_page: query.per_page,
        search: None,
        start_date: None,
        end_date: None,
        tenant_id: query.tenant_id,
        order_by: None,
        order_type: None,
    };

    let response = GescomService::list_alerts(
        db,
        &claims.sub,
        &claims.tenant_id,
        query.is_read,
        query.alert_type,
        params,
    ).await?;

    Ok(Json(PaginatedAlertResponse {
        data: response.data,
        total: response.total,
        page: response.page,
        per_page: response.per_page,
        total_pages: response.total_pages,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/alerts/{id}/read",
    params(
        ("id" = String, Path, description = "ID UUID de l'alerte")
    ),
    responses(
        (status = 200, description = "Alerte marquée comme lue.", body = AlertResponse),
        (status = 401, description = "Authentification requise."),
        (status = 404, description = "Alerte introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Alertes"
)]
pub async fn mark_alert_read(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<AlertResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_manage_alert").await?;

    let alert = GescomService::mark_alert_read(db, &id, &claims.sub, &claims.tenant_id).await?;
    Ok(Json(alert))
}

#[utoipa::path(
    post,
    path = "/api/v1/alerts/read-all",
    responses(
        (status = 200, description = "Toutes les alertes ont été marquées comme lues."),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Alertes"
)]
pub async fn mark_all_alerts_read(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    let count = GescomService::mark_all_alerts_read(db, &claims.sub, &claims.tenant_id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "rows_affected": count,
        "message": "Toutes les alertes ont été marquées comme lues."
    })))
}

// --- Sync Logs Endpoints ---

#[utoipa::path(
    post,
    path = "/api/v1/sync/logs",
    request_body = CreateSyncLogPayload,
    responses(
        (status = 201, description = "Log de sync créé avec succès.", body = SyncLogResponse),
        (status = 400, description = "Données invalides."),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Synchronisation"
)]
pub async fn create_sync_log(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateSyncLogPayload>,
) -> Result<Json<SyncLogResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_manage_sync_log").await?;

    let target_tenant_id = payload.tenant_id.as_deref().unwrap_or(&claims.tenant_id).to_string();
    require_tenant_access(db, &claims.tenant_id, &target_tenant_id, &claims.sub, "create").await?;

    let sync_log = GescomService::create_sync_log(db, &claims.sub, &target_tenant_id, payload).await?;
    Ok(Json(sync_log))
}

#[utoipa::path(
    get,
    path = "/api/v1/sync/logs",
    params(ListSyncLogsQuery),
    responses(
        (status = 200, description = "Liste paginée des logs de synchronisation.", body = PaginatedSyncLogResponse),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Synchronisation"
)]
pub async fn list_sync_logs(
    Query(query): Query<ListSyncLogsQuery>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<PaginatedSyncLogResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_sync_log").await?;

    let params = crate::utils::pagination::PaginationParams {
        page: query.page,
        per_page: query.per_page,
        search: None,
        start_date: query.start_date,
        end_date: query.end_date,
        tenant_id: query.tenant_id,
        order_by: None,
        order_type: None,
    };

    let response = GescomService::list_sync_logs(
        db,
        &claims.sub,
        &claims.tenant_id,
        query.device_id,
        params,
    ).await?;

    Ok(Json(PaginatedSyncLogResponse {
        data: response.data,
        total: response.total,
        page: response.page,
        per_page: response.per_page,
        total_pages: response.total_pages,
    }))
}
