use axum::{
    Json, Extension, extract::State
};
use std::sync::Arc;
use crate::{
    AppState,
    errors::ApiError,
    middleware::auth::Claims,
    dtos::tenant_dto::{UpdateTenantPayload, SetTenantTwoFactorPayload, TenantResponse},
    services::tenant_service::TenantService,
    utils::auth::require_permission,
};

#[utoipa::path(
    get,
    path = "/api/v1/admin/tenant",
    responses(
        (status = 200, description = "Détails du tenant récupérés avec succès.", body = TenantResponse),
        (status = 401, description = "Authentification requise ou token JWT invalide.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Tenant"
)]
pub async fn get_tenant(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<TenantResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    let response = TenantService::get_tenant(db, &claims.tenant_id).await?;
    Ok(Json(response))
}

#[utoipa::path(
    put,
    path = "/api/v1/admin/tenant",
    request_body(
        content = UpdateTenantPayload,
        description = "Champs à modifier pour le tenant",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Tenant modifié avec succès.", body = TenantResponse),
        (status = 401, description = "Authentification requise ou token JWT invalide."),
        (status = 403, description = "Permissions insuffisantes.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Tenant"
)]
pub async fn update_tenant(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateTenantPayload>,
) -> Result<Json<TenantResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    // Check permission
    require_permission(db, &claims.sub, "can_update_tenant").await?;

    let response = TenantService::update_tenant(db, &claims.tenant_id, payload).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/tenant/two-factor",
    request_body(
        content = SetTenantTwoFactorPayload,
        description = "Activation ou désactivation du Two-Factor pour le tenant",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Paramètre 2FA du tenant mis à jour avec succès.", body = TenantResponse),
        (status = 401, description = "Authentification requise ou token JWT invalide."),
        (status = 403, description = "Permissions insuffisantes.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Tenant"
)]
pub async fn set_tenant_two_factor(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SetTenantTwoFactorPayload>,
) -> Result<Json<TenantResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("La base de données n'est pas disponible".to_string())
    })?;

    // Check permission
    require_permission(db, &claims.sub, "can_set_tenant_two_factor").await?;

    let response = TenantService::set_two_factor(db, &claims.tenant_id, payload.two_factor_enabled).await?;
    Ok(Json(response))
}
