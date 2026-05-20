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
    dtos::category_dto::{CreateCategoryPayload, UpdateCategoryPayload, CategoryResponse},
    services::category_service::CategoryService,
    utils::auth::require_permission,
};

#[derive(Deserialize, IntoParams)]
pub struct ListCategoriesQuery {
    pub tenant_id: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/categories",
    params(ListCategoriesQuery),
    responses(
        (status = 200, description = "Liste des catégories récupérée avec succès.", body = Vec<CategoryResponse>),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Categories"
)]
pub async fn list_categories(
    Query(query): Query<ListCategoriesQuery>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CategoryResponse>>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_category").await?;

    let categories = CategoryService::list_categories(db, &claims.sub, &claims.tenant_id, query.tenant_id).await?;
    Ok(Json(categories))
}

#[utoipa::path(
    get,
    path = "/api/v1/categories/{id}",
    params(
        ("id" = String, Path, description = "Identifiant UUID de la catégorie")
    ),
    responses(
        (status = 200, description = "Détails de la catégorie.", body = CategoryResponse),
        (status = 401, description = "Authentification requise."),
        (status = 404, description = "Catégorie introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Categories"
)]
pub async fn get_category(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_category").await?;

    let category = CategoryService::get_category(db, &id, &claims.sub, &claims.tenant_id).await?;
    Ok(Json(category))
}

#[derive(Deserialize, IntoParams)]
pub struct CreateCategoryQuery {
    pub target_tenant_id: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/categories",
    params(CreateCategoryQuery),
    request_body(
        content = CreateCategoryPayload,
        description = "Champs nécessaires pour la création d'une catégorie",
        content_type = "application/json"
    ),
    responses(
        (status = 201, description = "Catégorie créée avec succès.", body = CategoryResponse),
        (status = 400, description = "Requête invalide."),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Categories"
)]
pub async fn create_category(
    Query(query): Query<CreateCategoryQuery>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCategoryPayload>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_create_category").await?;

    let category = CategoryService::create_category(db, &claims.sub, &claims.tenant_id, query.target_tenant_id, payload).await?;
    Ok(Json(category))
}

#[utoipa::path(
    put,
    path = "/api/v1/categories/{id}",
    params(
        ("id" = String, Path, description = "Identifiant UUID de la catégorie")
    ),
    request_body(
        content = UpdateCategoryPayload,
        description = "Champs à modifier",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Catégorie modifiée.", body = CategoryResponse),
        (status = 400, description = "Requête invalide."),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Categories"
)]
pub async fn update_category(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateCategoryPayload>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_update_category").await?;

    let category = CategoryService::update_category(db, &id, &claims.sub, &claims.tenant_id, payload).await?;
    Ok(Json(category))
}

#[utoipa::path(
    delete,
    path = "/api/v1/categories/{id}",
    params(
        ("id" = String, Path, description = "Identifiant UUID de la catégorie")
    ),
    responses(
        (status = 200, description = "Catégorie supprimée avec succès.", body = CategoryResponse),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Categories"
)]
pub async fn delete_category(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_delete_category").await?;

    let response = CategoryService::delete_category(db, &id, &claims.sub, &claims.tenant_id).await?;
    Ok(Json(response))
}
