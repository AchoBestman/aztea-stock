use crate::{
    AppState,
    dtos::subscription_dto::{
        CreateSubscriptionPayload, SubscriptionResponse, UpdateSubscriptionStatusPayload,
    },
    errors::ApiError,
    middleware::auth::Claims,
    models::tenant,
    services::subscription_service::SubscriptionService,
    utils::{auth::require_permission, pagination::PaginationParams},
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use sea_orm::EntityTrait;
use std::sync::Arc;

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
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Base de données indisponible".to_string()))?;

    require_permission(db, &claims.sub, "can_manage_subscriptions").await?;

    let sub = SubscriptionService::create_subscription(&state, payload).await?;
    Ok(Json(sub))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/subscriptions",
    params(PaginationParams),
    responses(
        (status = 200, description = "Liste paginée des abonnements.", body = PaginatedSubscriptionResponse),
    ),
    security(("bearerAuth" = [])),
    tag = "Admin - Subscriptions"
)]
pub async fn list_subscriptions(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<crate::dtos::subscription_dto::PaginatedSubscriptionResponse>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Base de données indisponible".to_string()))?;

    // Determine if caller is a system tenant — if not, scope to their own tenant_id
    let caller_tenant = tenant::Entity::find_by_id(&claims.tenant_id)
        .one(db)
        .await?
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
    Ok(Json(
        crate::dtos::subscription_dto::PaginatedSubscriptionResponse {
            data: subs.data,
            total: subs.total,
            page: subs.page,
            per_page: subs.per_page,
            total_pages: subs.total_pages,
        },
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/admin/subscriptions/{id}",
    params(("id" = String, Path, description = "Subscription ID")),
    responses(
        (status = 200, description = "Abonnement supprimé.", body = serde_json::Value),
    ),
    security(("bearerAuth" = [])),
    tag = "Admin - Subscriptions"
)]
pub async fn delete_subscription(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Path(subscription_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Base de données indisponible".to_string()))?;
    require_permission(db, &claims.sub, "can_manage_subscriptions").await?;
    SubscriptionService::delete_subscription(db, &subscription_id, &claims.sub, &claims.tenant_id)
        .await?;
    Ok(Json(
        serde_json::json!({"success": true, "message": "Abonnement supprimé."}),
    ))
}

#[utoipa::path(
    patch,
    path = "/api/v1/admin/subscriptions/{id}/status",
    request_body = UpdateSubscriptionStatusPayload,
    params(("id" = String, Path, description = "Subscription ID")),
    responses(
        (status = 200, description = "Statut mis à jour.", body = SubscriptionResponse),
    ),
    security(("bearerAuth" = [])),
    tag = "Admin - Subscriptions"
)]
pub async fn update_subscription_status(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Path(subscription_id): Path<String>,
    Json(payload): Json<UpdateSubscriptionStatusPayload>,
) -> Result<Json<SubscriptionResponse>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Base de données indisponible".to_string()))?;
    require_permission(db, &claims.sub, "can_manage_subscriptions").await?;
    let sub = SubscriptionService::update_subscription_status(
        db,
        &subscription_id,
        payload,
        &claims.sub,
        &claims.tenant_id,
    )
    .await?;
    Ok(Json(sub))
}

#[utoipa::path(
    put,
    path = "/api/v1/admin/subscriptions/{id}",
    request_body = UpdateSubscriptionPayload,
    params(("id" = String, Path, description = "Subscription ID")),
    responses(
        (status = 200, description = "Abonnement mis à jour.", body = SubscriptionResponse),
    ),
    security(("bearerAuth" = [])),
    tag = "Admin - Subscriptions"
)]
pub async fn update_subscription(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Path(subscription_id): Path<String>,
    Json(payload): Json<crate::dtos::subscription_dto::UpdateSubscriptionPayload>,
) -> Result<Json<SubscriptionResponse>, ApiError> {
    let sub = SubscriptionService::update_subscription(
        &state,
        &subscription_id,
        payload,
        &claims.sub,
        &claims.tenant_id,
    )
    .await?;
    Ok(Json(sub))
}
