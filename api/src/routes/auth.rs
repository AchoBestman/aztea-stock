use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use jsonwebtoken::{encode, Header, EncodingKey};

use crate::errors::ApiError;
use crate::models::{user, tenant, role, permission, user_role, role_permission};

#[derive(Deserialize, ToSchema)]
pub struct LoginPayload {
    /// User email address
    pub email: String,
    /// User password
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String, // Comma-separated list of roles
    pub tenant_id: String,
    pub tenant_name: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub user: UserProfile,
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
        ApiError::Database(sea_orm::DbErr::Custom("Connexion base de données indisponible".to_string()))
    })?;

    // 1. Retrieve the user by email
    let user_model = user::Entity::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(db)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Email ou mot de passe incorrect".to_string()))?;

    if user_model.is_active == Some(false) {
        return Err(ApiError::Unauthorized("Votre compte est inactif".to_string()));
    }

    // 2. Verify password with bcrypt
    bcrypt::verify(&payload.password, &user_model.password_hash)
        .map_err(|_| ApiError::Unauthorized("Email ou mot de passe incorrect".to_string()))
        .and_then(|valid| {
            if valid {
                Ok(())
            } else {
                Err(ApiError::Unauthorized("Email ou mot de passe incorrect".to_string()))
            }
        })?;

    // 3. Retrieve user's roles
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

    // 4. Retrieve Tenant
    let tenant_model = tenant::Entity::find_by_id(&user_model.tenant_id)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Tenant introuvable".to_string()))?;

    if tenant_model.is_active == Some(false) {
        return Err(ApiError::Unauthorized("Le compte de votre entreprise est inactif".to_string()));
    }

    // 5. Generate JWT tokens
    let role_names: Vec<String> = roles.iter().map(|r| r.name.clone()).collect();
    let role_str = role_names.join(",");
    let perm_names: Vec<String> = permissions.iter().map(|p| p.name.clone()).collect();

    let expires_in = 3600; // 1 hour
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
    ).map_err(|e| ApiError::Internal(format!("Failed to sign JWT: {}", e)))?;

    let refresh_token = uuid::Uuid::new_v4().to_string();

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        expires_in: expires_in as u64,
        user: UserProfile {
            id: user_model.id,
            name: user_model.name,
            email: user_model.email,
            role: role_str,
            tenant_id: tenant_model.id,
            tenant_name: tenant_model.name,
            roles: role_names,
            permissions: perm_names,
        },
    }))
}

pub fn router() -> Router<std::sync::Arc<crate::AppState>> {
    Router::new().route("/login", post(login))
}
