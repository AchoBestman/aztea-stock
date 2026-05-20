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
    dtos::product_dto::{CreateProductPayload, UpdateProductPayload, ProductResponse, PaginatedProductResponse},
    services::product_service::ProductService,
    utils::auth::require_permission,
};

#[derive(Deserialize, IntoParams)]
pub struct ListProductsQuery {
    pub tenant_id: Option<String>,
    pub category_id: Option<String>,
    pub is_active: Option<bool>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub search: Option<String>,
    pub order_by: Option<String>,
    pub order_type: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/products",
    request_body = CreateProductPayload,
    responses(
        (status = 201, description = "Produit créé avec succès.", body = ProductResponse),
        (status = 400, description = "Données invalides ou code-barres déjà utilisé."),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Products"
)]
pub async fn create_product(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateProductPayload>,
) -> Result<Json<ProductResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_manage_product").await?;

    let product = ProductService::create_product(
        db,
        &claims.sub,
        &claims.tenant_id,
        payload.tenant_id_override(&claims.tenant_id),
        payload,
    ).await?;

    Ok(Json(product))
}

#[utoipa::path(
    get,
    path = "/api/v1/products",
    params(ListProductsQuery),
    responses(
        (status = 200, description = "Liste des produits récupérée avec succès.", body = PaginatedProductResponse),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Products"
)]
pub async fn list_products(
    Query(query): Query<ListProductsQuery>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<PaginatedProductResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_product").await?;

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

    let products = ProductService::list_products(
        db,
        &claims.sub,
        &claims.tenant_id,
        query.tenant_id,
        query.category_id,
        query.is_active,
        pagination_params,
    ).await?;

    Ok(Json(PaginatedProductResponse {
        data: products.data,
        total: products.total,
        page: products.page,
        per_page: products.per_page,
        total_pages: products.total_pages,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/products/{id}",
    params(
        ("id" = String, Path, description = "Identifiant UUID du produit")
    ),
    responses(
        (status = 200, description = "Détails du produit.", body = ProductResponse),
        (status = 401, description = "Authentification requise."),
        (status = 404, description = "Produit introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Products"
)]
pub async fn get_product(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ProductResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_product").await?;

    let product = ProductService::get_product(db, &id, &claims.sub, &claims.tenant_id).await?;
    Ok(Json(product))
}

#[utoipa::path(
    put,
    path = "/api/v1/products/{id}",
    params(
        ("id" = String, Path, description = "Identifiant UUID du produit")
    ),
    request_body = UpdateProductPayload,
    responses(
        (status = 200, description = "Produit mis à jour avec succès.", body = ProductResponse),
        (status = 400, description = "Données invalides."),
        (status = 401, description = "Authentification requise."),
        (status = 404, description = "Produit introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Products"
)]
pub async fn update_product(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateProductPayload>,
) -> Result<Json<ProductResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_manage_product").await?;

    let product = ProductService::update_product(db, &id, &claims.sub, &claims.tenant_id, payload).await?;
    Ok(Json(product))
}

#[utoipa::path(
    delete,
    path = "/api/v1/products/{id}",
    params(
        ("id" = String, Path, description = "Identifiant UUID du produit")
    ),
    responses(
        (status = 200, description = "Produit supprimé avec succès.", body = ProductResponse),
        (status = 401, description = "Authentification requise."),
        (status = 404, description = "Produit introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Products"
)]
pub async fn delete_product(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ProductResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_manage_product").await?;

    let product = ProductService::delete_product(db, &id, &claims.sub, &claims.tenant_id).await?;
    Ok(Json(product))
}

trait TenantOverride {
    fn tenant_id_override(&self, _caller: &str) -> Option<String> {
        None
    }
}
impl TenantOverride for CreateProductPayload {}
