use crate::{
    AppState,
    dtos::user_dto::{
        CreateUserPayload, SendPasswordResetPayload, SetUserPasswordPayload,
        SetUserTwoFactorPayload, UserResponse,
    },
    errors::ApiError,
    middleware::auth::Claims,
    services::user_service::UserService,
    utils::auth::require_permission,
};
use axum::{
    Extension, Json,
    extract::{Query, State},
};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
pub struct ListUsersQuery {
    /// Filtrer par tenant (réservé au tenant système)
    pub tenant_id: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    params(ListUsersQuery),
    responses(
        (status = 200, description = "Liste des utilisateurs récupérée avec succès.", body = Vec<UserResponse>),
        (status = 401, description = "Authentification requise ou token JWT invalide."),
        (status = 403, description = "Permissions insuffisantes.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Users"
)]
pub async fn list_users(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<Vec<UserResponse>>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("La base de données n'est pas disponible".to_string()))?;

    // Check permission (either manage users or read user info)
    if require_permission(db, &claims.sub, "can_manage_tenant_users")
        .await
        .is_err()
    {
        require_permission(db, &claims.sub, "can_read_user").await?;
    }

    let users =
        UserService::list_users(db, &claims.sub, &claims.tenant_id, query.tenant_id).await?;
    Ok(Json(users))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/users",
    request_body(
        content = CreateUserPayload,
        description = "Champs obligatoires pour inviter/créer un utilisateur",
        content_type = "application/json"
    ),
    responses(
        (status = 201, description = "Utilisateur créé avec succès et invitation envoyée par email.", body = UserResponse),
        (status = 401, description = "Authentification requise ou token JWT invalide."),
        (status = 403, description = "Permissions insuffisantes.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Users"
)]
pub async fn create_user(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<Json<UserResponse>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("La base de données n'est pas disponible".to_string()))?;

    // Check permission (either manage users or create user)
    if require_permission(db, &claims.sub, "can_manage_tenant_users")
        .await
        .is_err()
    {
        require_permission(db, &claims.sub, "can_create_user").await?;
    }

    let user = UserService::create_user(&state, &claims.sub, &claims.tenant_id, payload).await?;
    Ok(Json(user))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/users/set-two-factor",
    request_body(
        content = SetUserTwoFactorPayload,
        description = "Champs pour configurer le 2FA d'un utilisateur",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "2FA mis à jour.", body = UserResponse),
        (status = 401, description = "Authentification requise ou token JWT invalide."),
        (status = 403, description = "Permissions insuffisantes.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Users"
)]
pub async fn set_user_two_factor(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SetUserTwoFactorPayload>,
) -> Result<Json<UserResponse>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("La base de données n'est pas disponible".to_string()))?;

    // Check can_manage_two_factor_for_user permission
    require_permission(db, &claims.sub, "can_manage_two_factor_for_user").await?;

    // Load target user to find their tenant
    let target_user = crate::repositories::user_repository::UserRepository::find_by_id_global(
        db,
        &payload.user_id,
    )
    .await?
    .ok_or_else(|| ApiError::NotFound("Utilisateur introuvable".to_string()))?;

    // Cross-tenant check: if target user is in a different tenant, caller must be system tenant
    crate::utils::auth::require_tenant_access(
        db,
        &claims.tenant_id,
        &target_user.tenant_id,
        &claims.sub,
        "update",
    )
    .await?;

    let user = UserService::set_two_factor(
        db,
        &target_user.tenant_id,
        &payload.user_id,
        payload.two_factor_enabled,
    )
    .await?;
    Ok(Json(user))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/users/password",
    request_body(
        content = SetUserPasswordPayload,
        description = "Champs pour définir directement le mot de passe d'un utilisateur",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Mot de passe mis à jour.", body = UserResponse),
        (status = 401, description = "Authentification requise ou token JWT invalide."),
        (status = 403, description = "Permissions insuffisantes.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Admin - Users"
)]
pub async fn set_user_password(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SetUserPasswordPayload>,
) -> Result<Json<UserResponse>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("La base de données n'est pas disponible".to_string()))?;

    // Check permission
    require_permission(db, &claims.sub, "can_set_tenant_password").await?;

    let user =
        UserService::set_password(db, &claims.tenant_id, &payload.user_id, &payload.password)
            .await?;
    Ok(Json(user))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/users/reset-password-request",
    request_body(
        content = SendPasswordResetPayload,
        description = "Email de l'utilisateur à réinitialiser",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Code envoyé par email avec succès."),
        (status = 401, description = "Authentification requise ou token JWT invalide."),
        (status = 403, description = "Permissions insuffisantes.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Auth"
)]
pub async fn send_user_reset(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SendPasswordResetPayload>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("La base de données n'est pas disponible".to_string()))?;

    // Check permission
    require_permission(db, &claims.sub, "can_send_tenant_password_reset").await?;

    UserService::send_password_reset(&state, &claims.tenant_id, &payload.email).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Email de réinitialisation envoyé avec succès."
    })))
}
