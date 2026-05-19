use axum::{
    routing::get,
    Router, Json, Extension, extract::{Path, State}
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use crate::{AppState, errors::ApiError, middleware::auth::Claims};

#[derive(Serialize, Deserialize, ToSchema, sqlx::FromRow, Debug, Clone)]
pub struct Role {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateRolePayload {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateRolePayload {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct DeleteRoleResponse {
    pub success: bool,
    pub message: String,
}

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

    let roles = sqlx::query_as::<_, Role>(
        "SELECT id, tenant_id, name, description FROM roles WHERE tenant_id = $1"
    )
    .bind(&claims.tenant_id)
    .fetch_all(pool)
    .await?;

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

    let role = sqlx::query_as::<_, Role>(
        "SELECT id, tenant_id, name, description FROM roles WHERE id = $1 AND tenant_id = $2"
    )
    .bind(&id)
    .bind(&claims.tenant_id)
    .fetch_optional(pool)
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

    // Check uniqueness
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM roles WHERE tenant_id = $1 AND name = $2"
    )
    .bind(&claims.tenant_id)
    .bind(&payload.name)
    .fetch_one(pool)
    .await?;

    if count.0 > 0 {
        return Err(ApiError::BadRequest("Un rôle avec ce nom existe déjà pour ce tenant.".to_string()));
    }

    let role_id = uuid::Uuid::new_v4().to_string();
    let role = sqlx::query_as::<_, Role>(
        "INSERT INTO roles (id, tenant_id, name, description) VALUES ($1, $2, $3, $4) RETURNING id, tenant_id, name, description"
    )
    .bind(&role_id)
    .bind(&claims.tenant_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .fetch_one(pool)
    .await?;

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

    // Check if role exists
    let role_exists: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM roles WHERE id = $1 AND tenant_id = $2"
    )
    .bind(&id)
    .bind(&claims.tenant_id)
    .fetch_one(pool)
    .await?;

    if role_exists.0 == 0 {
        return Err(ApiError::NotFound("Rôle introuvable".to_string()));
    }

    // Check uniqueness of name if changed
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM roles WHERE tenant_id = $1 AND name = $2 AND id != $3"
    )
    .bind(&claims.tenant_id)
    .bind(&payload.name)
    .bind(&id)
    .fetch_one(pool)
    .await?;

    if count.0 > 0 {
        return Err(ApiError::BadRequest("Un rôle avec ce nom existe déjà pour ce tenant.".to_string()));
    }

    let role = sqlx::query_as::<_, Role>(
        "UPDATE roles SET name = $1, description = $2 WHERE id = $3 AND tenant_id = $4 RETURNING id, tenant_id, name, description"
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&id)
    .bind(&claims.tenant_id)
    .fetch_one(pool)
    .await?;

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

    let rows_affected = sqlx::query(
        "DELETE FROM roles WHERE id = $1 AND tenant_id = $2"
    )
    .bind(&id)
    .bind(&claims.tenant_id)
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(ApiError::NotFound("Rôle introuvable".to_string()));
    }

    Ok(Json(DeleteRoleResponse {
        success: true,
        message: "Rôle supprimé avec succès.".to_string(),
    }))
}

pub fn router() -> Router<std::sync::Arc<crate::AppState>> {
    Router::new()
        .route("/roles", get(list_roles).post(create_role))
        .route("/roles/:id", get(get_role).put(update_role).delete(delete_role))
}
