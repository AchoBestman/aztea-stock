use crate::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,       // user_id
    pub tenant_id: String, // tenant_id
    pub role: String,
    pub exp: usize,
}

pub async fn extract_tenant(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    // Allow public endpoints to skip authentication
    let is_public_auth = path.starts_with("/api/v1/auth/login")
        || path.starts_with("/api/v1/auth/forgot-password")
        || path.starts_with("/api/v1/auth/reset-password")
        || path.starts_with("/api/v1/auth/verify-otp");

    if is_public_auth
        || path.starts_with("/api/v1/health")
        || path.starts_with("/api/v1/license/verify")
        || path.starts_with("/swagger-ui")
        || path.starts_with("/api-docs")
    {
        return Ok(next.run(req).await);
    }

    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?
    .claims;

    // Check if the tenant is active
    if let Some(db) = &state.db {
        use sea_orm::EntityTrait;
        let tenant_model = crate::models::tenant::Entity::find_by_id(&claims.tenant_id)
            .one(db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(t) = tenant_model {
            if t.is_active == Some(false) {
                return Err(StatusCode::UNAUTHORIZED);
            }
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
