use crate::{
    AppState,
    dtos::tenant_dto::{
        CreateTenantPayload, PaginatedTenantResponse, SetTenantTwoFactorPayload, TenantResponse,
        UpdateTenantPayload,
    },
    errors::ApiError,
    middleware::auth::Claims,
    services::tenant_service::TenantService,
    utils::auth::{check_permission, require_permission},
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use std::sync::Arc;

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct UpdateTenantQuery {
    pub tenant_id: Option<String>,
}

/// Paramètres de filtrage pour la liste des tenants
#[derive(serde::Deserialize, utoipa::IntoParams, Clone, Debug)]
pub struct ListTenantsQuery {
    /// Filtrer par type d'activité ('pharmacy', 'supermarket' ou 'both')
    pub business_type: Option<String>,
    /// Rechercher un motif sur les champs name, email, phone, country et address
    pub search: Option<String>,
    /// Filtrer par statut d'activation (valeurs acceptées : true, false, 1, 0)
    pub is_active: Option<String>,
    /// Filtrer par date de création supérieure ou égale (ex: ISO '2026-05-19' ou RFC3339 '2026-05-19T10:00:00Z')
    pub created_after: Option<String>,
    /// Filtrer par date de création inférieure ou égale (ex: ISO '2026-05-19' ou RFC3339 '2026-05-19T10:00:00Z')
    pub created_before: Option<String>,
    /// Filtrer par code pays (ISO2, ex: 'FR')
    pub country_code: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub order_by: Option<String>,
    pub order_type: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/tenants",
    request_body(
        content = CreateTenantPayload,
        description = "Champs pour créer un nouveau tenant",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Tenant créé avec succès.", body = TenantResponse),
        (status = 401, description = "Authentification requise ou token JWT invalide."),
        (status = 403, description = "Permissions insuffisantes.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Tenant"
)]
pub async fn create_tenant(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTenantPayload>,
) -> Result<Json<TenantResponse>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("La base de données n'est pas disponible".to_string()))?;

    // 1. Require can_create_tenant permission
    require_permission(db, &claims.sub, "can_create_tenant").await?;

    // 2. Load caller's tenant to verify it's the system tenant
    let caller_tenant =
        TenantService::load_tenant(db, &claims.tenant_id, "Tenant de l'utilisateur introuvable")
            .await?;

    if !caller_tenant.is_system {
        return Err(ApiError::Unauthorized(
            "Seul un utilisateur du tenant système est autorisé à créer un tenant.".to_string(),
        ));
    }

    let response = TenantService::create_tenant(db, payload).await?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/tenants",
    params(
        ListTenantsQuery
    ),
    params(ListTenantsQuery),
    responses(
        (status = 200, description = "Liste paginée de tous les tenants.", body = PaginatedTenantResponse),
        (status = 401, description = "Authentification requise ou token JWT invalide."),
        (status = 403, description = "Permissions insuffisantes.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Tenant"
)]
pub async fn list_tenants(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListTenantsQuery>,
) -> Result<Json<PaginatedTenantResponse>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("La base de données n'est pas disponible".to_string()))?;

    // 1. Require can_read_tenant permission
    require_permission(db, &claims.sub, "can_read_tenant").await?;

    // 2. Load caller's tenant to verify it's the system tenant
    let caller_tenant =
        TenantService::load_tenant(db, &claims.tenant_id, "Tenant de l'utilisateur introuvable")
            .await?;

    if !caller_tenant.is_system {
        return Err(ApiError::Unauthorized(
            "Seul un utilisateur du tenant système est autorisé à lister les tenants.".to_string(),
        ));
    }

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    let response = TenantService::list_tenants(
        db,
        query.business_type,
        query.search,
        query.is_active,
        query.created_after,
        query.created_before,
        query.country_code,
        page,
        per_page,
        query.order_by,
        query.order_type,
    )
    .await?;
    Ok(Json(response))
}

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
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("La base de données n'est pas disponible".to_string()))?;

    // Require can_read_tenant permission to see detail of own tenant
    require_permission(db, &claims.sub, "can_read_tenant").await?;

    let response = TenantService::get_tenant(db, &claims.tenant_id).await?;
    Ok(Json(response))
}

#[utoipa::path(
    put,
    path = "/api/v1/admin/tenant",
    params(
        UpdateTenantQuery
    ),
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
    Query(query): Query<UpdateTenantQuery>,
    Json(payload): Json<UpdateTenantPayload>,
) -> Result<Json<TenantResponse>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("La base de données n'est pas disponible".to_string()))?;

    // Check permission
    require_permission(db, &claims.sub, "can_update_tenant").await?;

    let target_tenant_id = query.tenant_id.as_deref().unwrap_or(&claims.tenant_id);

    let caller_has_credentials_permission =
        check_permission(db, &claims.sub, "can_update_tenant_credentials")
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

    let response = TenantService::update_tenant(
        db,
        &claims.sub,
        &claims.tenant_id,
        target_tenant_id,
        payload,
        caller_has_credentials_permission,
    )
    .await?;

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/tenants/{id}",
    responses(
        (status = 200, description = "Détails du tenant.", body = TenantResponse),
        (status = 401, description = "Authentification requise."),
        (status = 403, description = "Permissions insuffisantes."),
        (status = 404, description = "Tenant introuvable.")
    ),
    security(("bearerAuth" = [])),
    tag = "Admin - Tenant"
)]
pub async fn get_tenant_by_id(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<TenantResponse>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("La base de données n'est pas disponible".to_string()))?;

    require_permission(db, &claims.sub, "can_read_tenant").await?;

    let caller_tenant =
        TenantService::load_tenant(db, &claims.tenant_id, "Tenant de l'utilisateur introuvable")
            .await?;

    if !caller_tenant.is_system {
        return Err(ApiError::Unauthorized(
            "Seul un utilisateur du tenant système peut consulter un tenant par identifiant."
                .to_string(),
        ));
    }

    let response = TenantService::get_tenant(db, &id).await?;
    Ok(Json(response))
}

#[utoipa::path(
    delete,
    path = "/api/v1/admin/tenants/{id}",
    responses(
        (status = 200, description = "Tenant soft-deleté avec succès.", body = TenantResponse),
        (status = 401, description = "Authentification requise ou token JWT invalide."),
        (status = 403, description = "Permissions insuffisantes.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Tenant"
)]
pub async fn delete_tenant(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<TenantResponse>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("La base de données n'est pas disponible".to_string()))?;

    // 1. Require can_delete_tenant permission
    require_permission(db, &claims.sub, "can_delete_tenant").await?;

    let response = TenantService::delete_tenant(db, &claims.sub, &claims.tenant_id, &id).await?;
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
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("La base de données n'est pas disponible".to_string()))?;

    // 1. Check permission
    require_permission(db, &claims.sub, "can_manage_two_factor_for_tenant").await?;

    // 2. Determine target tenant
    let target_tenant_id = payload.tenant_id.as_deref().unwrap_or(&claims.tenant_id);

    // 3. Cross-tenant check
    crate::utils::auth::require_tenant_access(
        db,
        &claims.tenant_id,
        target_tenant_id,
        &claims.sub,
        "update",
    )
    .await?;

    let response =
        TenantService::set_two_factor(db, target_tenant_id, payload.two_factor_enabled).await?;
    Ok(Json(response))
}
