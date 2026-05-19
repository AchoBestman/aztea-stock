use axum::{
    Json, Extension, extract::{Path, State}
};
use std::sync::Arc;
use crate::{
    AppState,
    errors::ApiError,
    middleware::auth::Claims,
    models::role::{Role, CreateRolePayload, UpdateRolePayload, DeleteRoleResponse}
};

#[utoipa::path(
    get,
    path = "/api/v1/admin/roles",
    responses(
        (status = 200, description = "Liste des rôles récupérée avec succès.", body = Vec<Role>),
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
) -> Result<Json<Vec<Role>>, ApiError> {
    let pool = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    let roles = Role::list_by_tenant(pool, &claims.tenant_id).await?;
    Ok(Json(roles))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/roles/{id}",
    params(
        ("id" = String, Path, description = "L'identifiant UUID du rôle")
    ),
    responses(
        (status = 200, description = "Détails du rôle récupérés avec succès.", body = Role),
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
) -> Result<Json<Role>, ApiError> {
    let pool = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    let role = Role::find_by_id(pool, &claims.tenant_id, &id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Rôle introuvable".to_string()))?;

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
        (status = 201, description = "Rôle créé avec succès.", body = Role),
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
) -> Result<Json<Role>, ApiError> {
    let pool = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    if Role::exists_by_name(pool, &claims.tenant_id, &payload.name).await? {
        return Err(ApiError::BadRequest("Un rôle avec ce nom existe déjà pour ce tenant.".to_string()));
    }

    let role = Role::create(
        pool,
        &claims.tenant_id,
        &payload.name,
        payload.description.as_deref()
    ).await?;

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
        (status = 200, description = "Rôle modifié avec succès.", body = Role),
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
) -> Result<Json<Role>, ApiError> {
    let pool = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    if !Role::exists_by_id(pool, &claims.tenant_id, &id).await? {
        return Err(ApiError::NotFound("Rôle introuvable".to_string()));
    }

    if Role::exists_by_name_exclude(pool, &claims.tenant_id, &payload.name, &id).await? {
        return Err(ApiError::BadRequest("Un rôle avec ce nom existe déjà pour ce tenant.".to_string()));
    }

    let role = Role::update(
        pool,
        &id,
        &claims.tenant_id,
        &payload.name,
        payload.description.as_deref()
    ).await?;

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
    let pool = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    let success = Role::delete(pool, &id, &claims.tenant_id).await?;
    if !success {
        return Err(ApiError::NotFound("Rôle introuvable".to_string()));
    }

    Ok(Json(DeleteRoleResponse {
        success: true,
        message: "Rôle supprimé avec succès.".to_string(),
    }))
}
