use axum::{
    Json, Extension, extract::{Path, State}
};
use std::sync::Arc;
use crate::{
    AppState,
    errors::ApiError,
    middleware::auth::Claims,
    dtos::{
        create_role_dto::CreateRolePayload,
        update_role_dto::UpdateRolePayload,
        response_role_dto::{RoleResponse, DeleteRoleResponse}
    },
    services::role_service::RoleService,
    utils::auth::require_permission,
};

#[utoipa::path(
    get,
    path = "/api/v1/admin/roles",
    responses(
        (status = 200, description = "Liste des rôles récupérée avec succès.", body = Vec<RoleResponse>),
        (status = 401, description = "Authentification requise ou token JWT invalide.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Roles"
)]
pub async fn list_roles(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<RoleResponse>>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_role").await?;

    let roles = RoleService::list_roles(db, &claims.tenant_id).await?;
    Ok(Json(roles))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/roles/{id}",
    params(
        ("id" = String, Path, description = "L'identifiant UUID du rôle")
    ),
    responses(
        (status = 200, description = "Détails du rôle récupérés avec succès.", body = RoleResponse),
        (status = 401, description = "Authentification requise ou token JWT invalide."),
        (status = 404, description = "Rôle introuvable pour ce tenant.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Roles"
)]
pub async fn get_role(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<RoleResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_read_role").await?;

    let role = RoleService::get_role(db, &id, &claims.tenant_id).await?;
    Ok(Json(role))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/roles",
    request_body(
        content = CreateRolePayload,
        description = "Champs nécessaires pour la création d'un rôle",
        content_type = "application/json"
    ),
    responses(
        (status = 201, description = "Rôle créé avec succès.", body = RoleResponse),
        (status = 400, description = "Requête invalide ou rôle déjà existant."),
        (status = 401, description = "Authentification requise ou token JWT invalide.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Roles"
)]
pub async fn create_role(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateRolePayload>,
) -> Result<Json<RoleResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_create_role").await?;

    let role = RoleService::create_role(db, &claims.tenant_id, payload).await?;
    Ok(Json(role))
}

#[utoipa::path(
    put,
    path = "/api/v1/admin/roles/{id}",
    params(
        ("id" = String, Path, description = "L'identifiant UUID du rôle")
    ),
    request_body(
        content = UpdateRolePayload,
        description = "Champs à modifier pour le rôle",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Rôle modifié avec succès.", body = RoleResponse),
        (status = 400, description = "Requête invalide ou nom de rôle déjà pris."),
        (status = 401, description = "Authentification requise ou token JWT invalide."),
        (status = 404, description = "Rôle introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Roles"
)]
pub async fn update_role(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateRolePayload>,
) -> Result<Json<RoleResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_update_role").await?;

    let role = RoleService::update_role(db, &id, &claims.tenant_id, payload).await?;
    Ok(Json(role))
}

#[utoipa::path(
    delete,
    path = "/api/v1/admin/roles/{id}",
    params(
        ("id" = String, Path, description = "L'identifiant UUID du rôle")
    ),
    responses(
        (status = 200, description = "Rôle supprimé avec succès.", body = DeleteRoleResponse),
        (status = 401, description = "Authentification requise ou token JWT invalide."),
        (status = 404, description = "Rôle introuvable.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Roles"
)]
pub async fn delete_role(
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<DeleteRoleResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_delete_role").await?;

    let response = RoleService::delete_role(db, &id, &claims.tenant_id).await?;
    Ok(Json(response))
}
