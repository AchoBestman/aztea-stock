use axum::{Extension, Json, extract::{State, Query}};
use std::sync::Arc;
use crate::{
    AppState,
    errors::ApiError,
    middleware::auth::Claims,
    dtos::license_dto::{ActivateLicensePayload, FullLicenseResponse, GenerateLicensePayload, LicenseResponse, LicenseStatusResponse},
    services::license_service::LicenseService,
    utils::{auth::require_permission, pagination::PaginationParams},
    models::tenant,
};
use sea_orm::EntityTrait;

#[utoipa::path(
    post,
    path = "/api/v1/admin/licenses",
    request_body = GenerateLicensePayload,
    responses(
        (status = 200, description = "Clé de licence générée avec succès.", body = FullLicenseResponse),
    ),
    security(("bearerAuth" = [])),
    tag = "Admin - Licenses"
)]
pub async fn generate_license(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<GenerateLicensePayload>,
) -> Result<Json<FullLicenseResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("Base de données indisponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_manage_licenses").await?;

    let lic = LicenseService::generate_license(db, payload).await?;
    Ok(Json(lic))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/licenses",
    params(PaginationParams),
    responses(
        (status = 200, description = "Liste paginée des licences.", body = PaginatedLicenseResponse),
    ),
    security(("bearerAuth" = [])),
    tag = "Admin - Licenses"
)]
pub async fn list_licenses(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<crate::dtos::license_dto::PaginatedLicenseResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("Base de données indisponible".to_string())
    })?;

    let caller_tenant = tenant::Entity::find_by_id(&claims.tenant_id)
        .one(db).await?
        .ok_or_else(|| ApiError::Unauthorized("Tenant introuvable".to_string()))?;

    let enforce_tenant_id = if caller_tenant.is_system {
        None // Can filter by params.tenant_id freely
    } else {
        Some(claims.tenant_id.clone())
    };

    if !caller_tenant.is_system {
        require_permission(db, &claims.sub, "can_read_licenses").await?;
    } else {
        require_permission(db, &claims.sub, "can_manage_licenses").await?;
    }

    let lics = LicenseService::list_licenses(db, params, enforce_tenant_id).await?;
    Ok(Json(crate::dtos::license_dto::PaginatedLicenseResponse {
        data: lics.data,
        total: lics.total,
        page: lics.page,
        per_page: lics.per_page,
        total_pages: lics.total_pages,
    }))
}

/// Route utilisable par un tenant pour consulter l'état de sa licence active
#[utoipa::path(
    get,
    path = "/api/v1/licenses/status",
    responses(
        (status = 200, description = "Statut de la licence active.", body = LicenseStatusResponse),
        (status = 404, description = "Aucune licence active trouvée."),
    ),
    security(("bearerAuth" = [])),
    tag = "Licenses"
)]
pub async fn get_license_status(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<LicenseStatusResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("Base de données indisponible".to_string())
    })?;

    let status = LicenseService::get_license_status(db, &claims.tenant_id).await?;
    Ok(Json(status))
}

#[utoipa::path(
    post,
    path = "/api/v1/licenses/activate",
    request_body = ActivateLicensePayload,
    responses(
        (status = 200, description = "Clé activée avec succès.", body = LicenseResponse),
    ),
    security(("bearerAuth" = [])),
    tag = "Licenses"
)]
pub async fn activate_license(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ActivateLicensePayload>,
) -> Result<Json<LicenseResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("Base de données indisponible".to_string())
    })?;

    let lic = LicenseService::activate_license(db, &claims.tenant_id, payload).await?;
    Ok(Json(lic))
}
