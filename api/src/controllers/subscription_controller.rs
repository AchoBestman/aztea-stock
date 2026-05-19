use axum::{Extension, Json, extract::{State, Query}};
use std::sync::Arc;
use crate::{
    AppState,
    errors::ApiError,
    middleware::auth::Claims,
    dtos::subscription_dto::{CreateSubscriptionPayload, SubscriptionResponse},
    services::subscription_service::SubscriptionService,
    utils::{auth::require_permission, pagination::PaginationParams},
    models::tenant,
};
use sea_orm::EntityTrait;

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
    path = "/api/v1/admin/subscriptions",
    params(PaginationParams),
    responses(
        (status = 200, description = "Liste paginée des abonnements."),
    ),
    security(("bearerAuth" = [])),
    tag = "Admin - Subscriptions"
)]
pub async fn list_subscriptions(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<crate::utils::pagination::PaginatedResponse<SubscriptionResponse>>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Internal("Base de données indisponible".to_string())
    })?;

    // Determine if caller is a system tenant — if not, scope to their own tenant_id
    let caller_tenant = tenant::Entity::find_by_id(&claims.tenant_id)
        .one(db).await?
        .ok_or_else(|| ApiError::Unauthorized("Tenant introuvable".to_string()))?;

    let enforce_tenant_id = if caller_tenant.is_system {
        // System admin can use params.tenant_id freely (including None = all)
        None
    } else {
        // Non-system tenant sees only their own subscriptions
        Some(claims.tenant_id.clone())
    };

    if !caller_tenant.is_system {
        require_permission(db, &claims.sub, "can_read_subscriptions").await?;
    } else {
        require_permission(db, &claims.sub, "can_manage_subscriptions").await?;
    }

    let subs = SubscriptionService::list_subscriptions(db, params, enforce_tenant_id).await?;
    Ok(Json(subs))
}
