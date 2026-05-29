use axum::{
    Extension, Json, Router,
    extract::State,
    routing::{get, post},
};
use jsonwebtoken::{EncodingKey, Header, encode};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::AppState;
use crate::dtos::user_dto::{UpdateProfilePayload, UserProfileResponse, UserProfileTenantResponse};
use crate::errors::ApiError;
use crate::middleware::auth::Claims;
use crate::models::{permission, role, role_permission, tenant, user, user_role};
use crate::services::user_service::UserService;

#[derive(Deserialize, ToSchema)]
pub struct LoginPayload {
    /// User email address
    pub email: String,
    /// User password
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ForgotPasswordPayload {
    pub email: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ResetPasswordPayload {
    pub email: String,
    pub otp_code: String,
    pub new_password: String,
}

#[derive(Serialize, ToSchema)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String, // Comma-separated list of roles
    pub tenant_id: String,
    pub tenant_name: String,
    pub tenant: UserProfileTenantResponse,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct VerifyOtpPayload {
    pub email: String,
    pub otp_code: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub requires_two_factor: bool,
    pub message: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub user: Option<UserProfile>,
}

async fn generate_login_response(
    db: &sea_orm::DatabaseConnection,
    state: &AppState,
    user_model: &user::Model,
    tenant_model: &tenant::Model,
) -> Result<LoginResponse, ApiError> {
    let user_roles = user_role::Entity::find()
        .filter(user_role::Column::UserId.eq(&user_model.id))
        .all(db)
        .await?;

    let role_ids: Vec<String> = user_roles.into_iter().map(|ur| ur.role_id).collect();

    let mut roles = Vec::new();
    let mut permissions = Vec::new();

    if !role_ids.is_empty() {
        roles = role::Entity::find()
            .filter(role::Column::Id.is_in(role_ids.clone()))
            .all(db)
            .await?;

        let role_perms = role_permission::Entity::find()
            .filter(role_permission::Column::RoleId.is_in(role_ids))
            .all(db)
            .await?;

        let perm_ids: Vec<String> = role_perms.into_iter().map(|rp| rp.permission_id).collect();
        if !perm_ids.is_empty() {
            permissions = permission::Entity::find()
                .filter(permission::Column::Id.is_in(perm_ids))
                .all(db)
                .await?;
        }
    }

    let role_names: Vec<String> = roles.iter().map(|r| r.name.clone()).collect();
    let role_str = role_names.join(",");
    let perm_names: Vec<String> = permissions.iter().map(|p| p.name.clone()).collect();

    let expires_in = 3600;
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(expires_in))
        .expect("valid timestamp")
        .timestamp();

    let claims = crate::middleware::auth::Claims {
        sub: user_model.id.clone(),
        tenant_id: user_model.tenant_id.clone(),
        role: role_str.clone(),
        exp: expiration as usize,
    };

    let access_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| ApiError::Internal(format!("Failed to sign JWT: {}", e)))?;

    let refresh_token = uuid::Uuid::new_v4().to_string();

    Ok(LoginResponse {
        requires_two_factor: false,
        message: None,
        access_token: Some(access_token),
        refresh_token: Some(refresh_token),
        expires_in: Some(expires_in as u64),
        user: Some(UserProfile {
            id: user_model.id.clone(),
            name: user_model.name.clone(),
            email: user_model.email.clone(),
            role: role_str,
            tenant_id: tenant_model.id.clone(),
            tenant_name: tenant_model.name.clone(),
            tenant: UserProfileTenantResponse::from_tenant(tenant_model),
            roles: role_names,
            permissions: perm_names,
        }),
    })
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body(
        content = LoginPayload,
        description = "Identifiants de connexion obligatoires.",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Connexion réussie. Retourne le jeton d'accès JWT et les informations de profil.", body = LoginResponse),
        (status = 400, description = "Format de requête invalide ou champs obligatoires manquants."),
        (status = 401, description = "Authentification échouée (identifiants incorrects).")
    ),
    tag = "Auth"
)]
pub async fn login(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<crate::AppState>>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Database(sea_orm::DbErr::Custom(
            "Connexion base de données indisponible".to_string(),
        ))
    })?;

    // 1. Retrieve the user by email
    let user_model = user::Entity::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(db)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Email ou mot de passe incorrect".to_string()))?;

    if user_model.is_active == Some(false) {
        return Err(ApiError::Unauthorized(
            "Votre compte est inactif".to_string(),
        ));
    }

    // 2. Verify password with bcrypt
    bcrypt::verify(&payload.password, &user_model.password_hash)
        .map_err(|_| ApiError::Unauthorized("Email ou mot de passe incorrect".to_string()))
        .and_then(|valid| {
            if valid {
                Ok(())
            } else {
                Err(ApiError::Unauthorized(
                    "Email ou mot de passe incorrect".to_string(),
                ))
            }
        })?;

    // 3. Retrieve Tenant
    let tenant_model = tenant::Entity::find_by_id(&user_model.tenant_id)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Tenant introuvable".to_string()))?;

    if tenant_model.is_active == Some(false) {
        return Err(ApiError::Unauthorized(
            "Le compte de votre entreprise est inactif".to_string(),
        ));
    }

    // 4. Check 2FA requirement
    let user_requires_2fa = user_model.two_factor_enabled;
    let tenant_requires_2fa = tenant_model.two_factor_enabled;

    if user_requires_2fa || tenant_requires_2fa {
        use rand::Rng;
        use sea_orm::ActiveModelTrait;

        // Generate 6-digit OTP
        let code: String = (0..6)
            .map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric) as char)
            .collect::<String>()
            .to_uppercase();

        let mut user_active: user::ActiveModel = user_model.clone().into();
        user_active.two_factor_code = sea_orm::Set(Some(code.clone()));
        user_active.two_factor_expires_at = sea_orm::Set(Some(
            (chrono::Utc::now() + chrono::Duration::minutes(5)).fixed_offset(),
        ));
        user_active.update(db).await?;

        // Send email
        crate::services::email_service::send_otp_email(
            &state,
            &tenant_model.id,
            &payload.email,
            &code,
        )
        .await
        .map_err(|e| ApiError::Internal(format!("Erreur d'envoi d'email: {}", e)))?;

        return Ok(Json(LoginResponse {
            requires_two_factor: true,
            message: Some(
                "Un code de vérification a été envoyé à votre adresse email.".to_string(),
            ),
            access_token: None,
            refresh_token: None,
            expires_in: None,
            user: None,
        }));
    }

    // 5. Generate JWT tokens
    let response = generate_login_response(db, &state, &user_model, &tenant_model).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/verify-otp",
    request_body(
        content = VerifyOtpPayload,
        description = "Email et code OTP de connexion",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Connexion réussie. Retourne le jeton d'accès.", body = LoginResponse),
        (status = 400, description = "Requête invalide ou code OTP expiré."),
        (status = 401, description = "Authentification échouée.")
    ),
    tag = "Auth"
)]
pub async fn verify_otp(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyOtpPayload>,
) -> Result<Json<LoginResponse>, ApiError> {
    let db = state.db.as_ref().ok_or_else(|| {
        ApiError::Database(sea_orm::DbErr::Custom(
            "Connexion base de données indisponible".to_string(),
        ))
    })?;

    // Find user
    let user_model = user::Entity::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(db)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Email ou code incorrect".to_string()))?;

    // Verify OTP
    if let Some(ref stored_code) = user_model.two_factor_code {
        if stored_code != &payload.otp_code {
            return Err(ApiError::Unauthorized(
                "Code de vérification incorrect".to_string(),
            ));
        }
    } else {
        return Err(ApiError::BadRequest(
            "Aucun code de vérification n'a été demandé".to_string(),
        ));
    }

    // Verify Expiration
    if let Some(expires_at) = user_model.two_factor_expires_at {
        if chrono::Utc::now().fixed_offset() > expires_at {
            return Err(ApiError::BadRequest(
                "Le code de vérification a expiré".to_string(),
            ));
        }
    } else {
        return Err(ApiError::BadRequest(
            "Le code de vérification a expiré".to_string(),
        ));
    }

    let tenant_model = tenant::Entity::find_by_id(&user_model.tenant_id)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Tenant introuvable".to_string()))?;

    if tenant_model.is_active == Some(false) {
        return Err(ApiError::Unauthorized(
            "Le compte de votre entreprise est inactif".to_string(),
        ));
    }

    // Clear OTP
    use sea_orm::ActiveModelTrait;
    let mut user_active: user::ActiveModel = user_model.clone().into();
    user_active.two_factor_code = sea_orm::Set(None);
    user_active.two_factor_expires_at = sea_orm::Set(None);
    user_active.update(db).await?;

    let response = generate_login_response(db, &state, &user_model, &tenant_model).await?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/profile",
    responses(
        (status = 200, description = "Profil de l'utilisateur connecté récupéré avec succès.", body = UserProfileResponse),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Auth"
)]
pub async fn get_profile(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("La base de données n'est pas disponible".to_string()))?;

    let profile = UserService::get_profile(db, &claims.sub).await?;
    Ok(Json(profile))
}

#[utoipa::path(
    put,
    path = "/api/v1/auth/profile",
    request_body(
        content = UpdateProfilePayload,
        description = "Champs pour modifier le profil",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Profil mis à jour avec succès.", body = UserProfileResponse),
        (status = 400, description = "Requête invalide."),
        (status = 401, description = "Authentification requise."),
        (status = 403, description = "Permissions insuffisantes.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Auth"
)]
pub async fn update_profile(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateProfilePayload>,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let profile =
        UserService::update_profile(&state, &claims.sub, &claims.tenant_id, payload).await?;
    Ok(Json(profile))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password",
    request_body(
        content = ForgotPasswordPayload,
        description = "Email pour demander la réinitialisation",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Code envoyé par email avec succès."),
        (status = 404, description = "Utilisateur introuvable.")
    ),
    tag = "Auth"
)]
pub async fn forgot_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ForgotPasswordPayload>,
) -> Result<Json<serde_json::Value>, ApiError> {
    UserService::send_public_password_reset(&state, &payload.email).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Email de réinitialisation envoyé avec succès."
    })))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/reset-password",
    request_body(
        content = ResetPasswordPayload,
        description = "Email, code OTP et nouveau mot de passe",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Mot de passe réinitialisé avec succès."),
        (status = 400, description = "Requête invalide ou code OTP expiré."),
        (status = 404, description = "Utilisateur introuvable.")
    ),
    tag = "Auth",
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResetPasswordPayload>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("La base de données n'est pas disponible".to_string()))?;

    UserService::public_reset_password(
        db,
        &payload.email,
        &payload.otp_code,
        &payload.new_password,
    )
    .await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Mot de passe réinitialisé avec succès."
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/device-key",
    responses(
        (status = 200, description = "Clé de chiffrement des terminaux.", body = serde_json::Value),
        (status = 401, description = "Authentification requise.")
    ),
    security(
        ("bearerAuth" = [])
    ),
    tag = "Auth"
)]
pub async fn get_device_key(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::Internal("La base de données n'est pas disponible".to_string()))?;

    // Authenticate/verify user exists
    let _user = user::Entity::find_by_id(&claims.sub)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Compte introuvable".to_string()))?;

    crate::utils::auth::require_permission(db, &claims.sub, "can_read_device_key").await?;

    let key = crate::utils::crypto::get_key_string();
    Ok(Json(serde_json::json!({
        "device_key": key
    })))
}

pub fn router() -> Router<std::sync::Arc<crate::AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/profile", get(get_profile).put(update_profile))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route("/verify-otp", post(verify_otp))
        .route("/device-key", get(get_device_key))
}
