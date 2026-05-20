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
    dtos::stock_dto::{
        CreateStockItemPayload, UpdateStockItemPayload, StockItemResponse, PaginatedStockItemResponse,
        CreateStockMovementPayload, StockMovementResponse, PaginatedStockMovementResponse
    },
    services::stock_service::StockService,
    utils::auth::require_permission,
};

#[derive(Deserialize, IntoParams)]
pub struct ListStockItemsQuery {
    pub tenant_id: Option<String>,
    pub category_id: Option<String>,
    pub is_low_stock: Option<bool>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub search: Option<String>,
    pub order_by: Option<String>,
    pub order_type: Option<String>,
}

#[derive(Deserialize, IntoParams)]
pub struct ListStockMovementsQuery {
    pub tenant_id: Option<String>,
    pub product_id: Option<String>,
    pub movement_type: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

// --- Stock Items ---

#[utoipa::path(
    post,
    path = "/api/v1/stock/items",
    request_body = CreateStockItemPayload,
    responses(
        (status = 201, description = "Fiche stock créée avec succès.", body = StockItemResponse),
        (status = 400, description = "Données invalides ou article déjà existant."),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Stock"
)]
pub async fn create_stock_item(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateStockItemPayload>,
) -> Result<Json<StockItemResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_manage_stock").await?;

    let item = StockService::create_stock_item(
        db,
        &claims.sub,
        &claims.tenant_id,
        payload,
    ).await?;

    Ok(Json(item))
}

#[utoipa::path(
    get,
    path = "/api/v1/stock/items",
    params(ListStockItemsQuery),
    responses(
        (status = 200, description = "Liste paginée des articles en stock.", body = PaginatedStockItemResponse),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Stock"
)]
pub async fn list_stock_items(
    Query(query): Query<ListStockItemsQuery>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<PaginatedStockItemResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_stock").await?;

    let pagination_params = crate::utils::pagination::PaginationParams {
        page: query.page,
        per_page: query.per_page,
        search: query.search,
        start_date: None,
        end_date: None,
        tenant_id: query.tenant_id.clone(),
        order_by: query.order_by,
        order_type: query.order_type,
    };

    let items = StockService::list_stock_items(
        db,
        &claims.sub,
        &claims.tenant_id,
        query.category_id,
        query.is_low_stock,
        pagination_params,
    ).await?;

    Ok(Json(PaginatedStockItemResponse {
        data: items.data,
        total: items.total,
        page: items.page,
        per_page: items.per_page,
        total_pages: items.total_pages,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/stock/items/{id}",
    params(
        ("id" = String, Path, description = "Identifiant UUID de la fiche stock")
    ),
    responses(
        (status = 200, description = "Détails de la fiche stock.", body = StockItemResponse),
        (status = 401, description = "Authentification requise."),
        (status = 404, description = "Fiche stock introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Stock"
)]
pub async fn get_stock_item(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<StockItemResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_stock").await?;

    let item = StockService::get_stock_item(db, &id, &claims.sub, &claims.tenant_id).await?;
    Ok(Json(item))
}

#[utoipa::path(
    put,
    path = "/api/v1/stock/items/{id}",
    params(
        ("id" = String, Path, description = "Identifiant UUID de la fiche stock")
    ),
    request_body = UpdateStockItemPayload,
    responses(
        (status = 200, description = "Fiche stock mise à jour avec succès.", body = StockItemResponse),
        (status = 400, description = "Données invalides."),
        (status = 401, description = "Authentification requise."),
        (status = 404, description = "Fiche stock introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Stock"
)]
pub async fn update_stock_item(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateStockItemPayload>,
) -> Result<Json<StockItemResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_manage_stock").await?;

    let item = StockService::update_stock_item(db, &id, &claims.sub, &claims.tenant_id, payload).await?;
    Ok(Json(item))
}

#[utoipa::path(
    delete,
    path = "/api/v1/stock/items/{id}",
    params(
        ("id" = String, Path, description = "Identifiant UUID de la fiche stock")
    ),
    responses(
        (status = 200, description = "Fiche stock supprimée avec succès."),
        (status = 401, description = "Authentification requise."),
        (status = 404, description = "Fiche stock introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Stock"
)]
pub async fn delete_stock_item(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_manage_stock").await?;

    StockService::delete_stock_item(db, &id, &claims.sub, &claims.tenant_id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Fiche stock supprimée avec succès."
    })))
}

// --- Stock Movements ---

#[utoipa::path(
    post,
    path = "/api/v1/stock/movements",
    request_body = CreateStockMovementPayload,
    responses(
        (status = 201, description = "Mouvement de stock enregistré avec succès.", body = StockMovementResponse),
        (status = 400, description = "Données invalides ou stock négatif."),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Stock"
)]
pub async fn create_stock_movement(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateStockMovementPayload>,
) -> Result<Json<StockMovementResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_manage_stock").await?;

    let movement = StockService::create_stock_movement(
        db,
        &claims.sub,
        &claims.tenant_id,
        payload,
    ).await?;

    Ok(Json(movement))
}

#[utoipa::path(
    get,
    path = "/api/v1/stock/movements",
    params(ListStockMovementsQuery),
    responses(
        (status = 200, description = "Liste paginée des mouvements de stock.", body = PaginatedStockMovementResponse),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Stock"
)]
pub async fn list_stock_movements(
    Query(query): Query<ListStockMovementsQuery>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<PaginatedStockMovementResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_stock").await?;

    let pagination_params = crate::utils::pagination::PaginationParams {
        page: query.page,
        per_page: query.per_page,
        search: None,
        start_date: query.start_date,
        end_date: query.end_date,
        tenant_id: query.tenant_id.clone(),
        order_by: None,
        order_type: None,
    };

    let movements = StockService::list_stock_movements(
        db,
        &claims.sub,
        &claims.tenant_id,
        query.product_id,
        query.movement_type,
        pagination_params,
    ).await?;

    Ok(Json(PaginatedStockMovementResponse {
        data: movements.data,
        total: movements.total,
        page: movements.page,
        per_page: movements.per_page,
        total_pages: movements.total_pages,
    }))
}
