use axum::{
    Json, Extension, extract::State
};
use std::sync::Arc;
use crate::{
    AppState,
    errors::ApiError,
    middleware::auth::Claims,
    services::permission_service::{PermissionService, GroupedPermissionsResponse},
    utils::auth::require_permission,
};

#[utoipa::path(
    get,
    path = "/api/v1/admin/permissions",
    responses(
        (status = 200, description = "Liste des permissions système groupées par module récupérée avec succès.", body = Vec<GroupedPermissionsResponse>),
        (status = 401, description = "Authentification requise."),
        (status = 403, description = "Permissions insuffisantes.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Roles"
)]
pub async fn list_permissions(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<GroupedPermissionsResponse>>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    // 1. Guard: only a user with "can_read_permission" permission can access this route
    require_permission(db, &claims.sub, "can_read_permission").await?;

    // 2. Fetch grouped permissions
    let permissions = PermissionService::list_grouped_permissions(db).await?;
    
    Ok(Json(permissions))
}
