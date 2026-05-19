use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct LoginPayload {
    /// User email address
    pub email: String,
    /// User password
    pub password: String,
    /// Associated subscription license key
    pub license_key: String,
}

#[derive(Serialize, ToSchema)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub role: String,
    pub tenant_id: String,
    pub tenant_name: String,
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
        description = "Identifiants de connexion et clé de licence obligatoires.",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Connexion réussie. Retourne le jeton d'accès JWT et les informations de profil.", body = LoginResponse),
        (status = 400, description = "Format de requête invalide ou champs obligatoires manquants."),
        (status = 401, description = "Authentification échouée (identifiants incorrects ou clé de licence invalide/suspendue).")
    ),
    tag = "Auth"
)]
pub async fn login(
    axum::extract::State(_state): axum::extract::State<std::sync::Arc<crate::AppState>>,
    Json(_payload): Json<LoginPayload>,
) -> Json<LoginResponse> {
    Json(LoginResponse {
        access_token: "mock_jwt_token".to_string(),
        refresh_token: "mock_refresh_token".to_string(),
        expires_in: 3600,
        user: UserProfile {
            id: "00000000-0000-0000-0000-000000000000".to_string(),
            name: "Jean Moukala".to_string(),
            role: "manager".to_string(),
            tenant_id: "00000000-0000-0000-0000-000000000000".to_string(),
            tenant_name: "Pharmacie ABC".to_string(),
        },
    })
}

pub fn router() -> Router<std::sync::Arc<crate::AppState>> {
    Router::new().route("/login", post(login))
}
