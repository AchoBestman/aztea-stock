use axum::{Extension, Json, extract::{State, Path}};
use std::sync::Arc;
use crate::{
    AppState,
    errors::ApiError,
    middleware::auth::Claims,
    dtos::license_dto::{ActivateLicensePayload, FullLicenseResponse, GenerateLicensePayload, LicenseResponse},
    services::license_service::LicenseService,
    utils::auth::require_permission,
};

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
    path = "/api/v1/admin/tenants/{tenant_id}/licenses",
    params(
        ("tenant_id" = String, Path, description = "ID of the tenant")
    ),
    responses(
        (status = 200, description = "Liste des licences.", body = Vec<LicenseResponse>),
    ),
    security(("bearerAuth" = [])),
    tag = "Admin - Licenses"
)]
pub async fn list_tenant_licenses(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Path(tenant_id): Path<String>,
) -> Result<Json<Vec<LicenseResponse>>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("Base de données indisponible".to_string())
    })?;

    if tenant_id != claims.tenant_id {
        require_permission(db, &claims.sub, "can_read_all_licenses").await?;
    } else {
        require_permission(db, &claims.sub, "can_read_licenses").await?;
    }

    let lics = LicenseService::list_tenant_licenses(db, &tenant_id).await?;
    Ok(Json(lics))
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
