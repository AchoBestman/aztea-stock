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

    // 1. Check permission
    require_permission(db, &claims.sub, "can_set_tenant_two_factor").await?;

    // 2. Load caller's tenant to check if they belong to the system tenant (is_system = true)
    let caller_tenant = crate::repositories::tenant_repository::TenantRepository::find_by_id(db, &claims.tenant_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Tenant de l'utilisateur introuvable".to_string()))?;

    // 3. Enforce the rule:
    // Determine the target tenant_id: default to caller's tenant_id if not specified in the payload.
    let target_tenant_id = payload.tenant_id.as_deref().unwrap_or(&claims.tenant_id);

    // User must belong to the target tenant, UNLESS they are an authorized user from the system tenant (is_system = true).
    if target_tenant_id != claims.tenant_id && !caller_tenant.is_system {
        return Err(ApiError::Unauthorized(
            "Vous n'êtes pas autorisé à modifier la double authentification pour un autre tenant.".to_string()
        ));
    }

    let response = TenantService::set_two_factor(db, target_tenant_id, payload.two_factor_enabled).await?;
    Ok(Json(response))
}
