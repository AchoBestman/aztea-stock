use axum::{Extension, Json, extract::{State, Path}};
use std::sync::Arc;
use crate::{
    AppState,
    errors::ApiError,
    middleware::auth::Claims,
    dtos::subscription_dto::{CreateSubscriptionPayload, SubscriptionResponse},
    services::subscription_service::SubscriptionService,
    utils::auth::require_permission,
};

#[utoipa::path(
    post,
    path = "/api/v1/admin/subscriptions",
    request_body = CreateSubscriptionPayload,
    responses(
        (status = 201, description = "Abonnement créé avec succès.", body = SubscriptionResponse),
    ),
    security(("bearerAuth" = [])),
    tag = "Admin - Subscriptions"
)]
pub async fn create_subscription(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateSubscriptionPayload>,
) -> Result<Json<SubscriptionResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("Base de données indisponible".to_string())
    })?;

    require_permission(db, &claims.sub, "can_manage_subscriptions").await?;

    let sub = SubscriptionService::create_subscription(db, payload).await?;
    Ok(Json(sub))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/tenants/{tenant_id}/subscriptions",
    params(
        ("tenant_id" = String, Path, description = "ID of the tenant")
    ),
    responses(
        (status = 200, description = "Liste des abonnements.", body = Vec<SubscriptionResponse>),
    ),
    security(("bearerAuth" = [])),
    tag = "Admin - Subscriptions"
)]
pub async fn list_tenant_subscriptions(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Path(tenant_id): Path<String>,
) -> Result<Json<Vec<SubscriptionResponse>>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("Base de données indisponible".to_string())
    })?;

    if tenant_id != claims.tenant_id {
        require_permission(db, &claims.sub, "can_read_all_subscriptions").await?;
    } else {
        require_permission(db, &claims.sub, "can_read_subscriptions").await?;
    }

    let subs = SubscriptionService::list_tenant_subscriptions(db, &tenant_id).await?;
    Ok(Json(subs))
}
