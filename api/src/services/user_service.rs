use sea_orm::DatabaseConnection;
use bcrypt::{hash, DEFAULT_COST};
use rand::Rng;

use crate::{
    AppState,
    errors::ApiError,
    repositories::user_repository::UserRepository,
    dtos::user_dto::{CreateUserPayload, UserResponse},
    services::email_service::send_password_reset_email,
};

pub struct UserService;

impl UserService {
    pub async fn list_users(
        db: &DatabaseConnection,
        tenant_id: &str,
    ) -> Result<Vec<UserResponse>, ApiError> {
        let models = UserRepository::find_all_by_tenant(db, tenant_id).await?;
        let mut responses = Vec::new();
        for m in models {
            let roles = UserRepository::get_user_roles(db, &m.id).await?;
            responses.push(Self::map_to_response(m, roles));
        }
        Ok(responses)
    }

    pub async fn create_user(
        state: &AppState,
        tenant_id: &str,
        payload: CreateUserPayload,
    ) -> Result<UserResponse, ApiError> {
        let db = state.db.as_ref().ok_or_else(|| {
            ApiError::Internal("Base de données indisponible".to_string())
        })?;

        // 1. Check if user already exists
        if UserRepository::find_by_email(db, &payload.email).await?.is_some() {
            return Err(ApiError::BadRequest("Un utilisateur avec cet email existe déjà".to_string()));
        }

        // 2. Generate a random password and hash it
        let temp_pass: String = (0..12)
            .map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric) as char)
            .collect();
        let password_hash = hash(&temp_pass, DEFAULT_COST)
            .map_err(|e| ApiError::Internal(format!("Erreur lors du hachage : {}", e)))?;

        // 3. Create the user
        let user_id = uuid::Uuid::new_v4().to_string();
        let mut user = UserRepository::create(
            db,
            &user_id,
            tenant_id,
            &payload.name,
            &payload.email,
            &password_hash,
        )
        .await?;

        // 4. Assign role
        UserRepository::assign_role(db, &user_id, &payload.role_id).await?;

        // 5. Generate validation code (6-digit alphanumeric or simple code) for reset
        let code: String = (0..6)
            .map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric) as char)
            .collect::<String>()
            .to_uppercase();

        user.two_factor_code = Some(code.clone());
        user.two_factor_expires_at = Some(
            (chrono::Utc::now() + chrono::Duration::hours(24)).fixed_offset(),
        );
        let user = UserRepository::update(db, user).await?;

        // 6. Send invitation/password reset email
        let _ = send_password_reset_email(state, tenant_id, &payload.email, &code).await;

        let roles = UserRepository::get_user_roles(db, &user.id).await?;
        Ok(Self::map_to_response(user, roles))
    }

    pub async fn set_two_factor(
        db: &DatabaseConnection,
        tenant_id: &str,
        user_id: &str,
        enabled: bool,
    ) -> Result<UserResponse, ApiError> {
        let mut user = UserRepository::find_by_id(db, user_id, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Utilisateur introuvable".to_string()))?;

        user.two_factor_enabled = enabled;
        let updated = UserRepository::update(db, user).await?;

        let roles = UserRepository::get_user_roles(db, &updated.id).await?;
        Ok(Self::map_to_response(updated, roles))
    }

    pub async fn send_password_reset(
        state: &AppState,
        tenant_id: &str,
        email: &str,
    ) -> Result<(), ApiError> {
        let db = state.db.as_ref().ok_or_else(|| {
            ApiError::Internal("Base de données indisponible".to_string())
        })?;

        let mut user = UserRepository::find_by_email_and_tenant(db, email, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Utilisateur introuvable pour ce tenant".to_string()))?;

        // Generate 6-digit random code
        let code: String = (0..6)
            .map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric) as char)
            .collect::<String>()
            .to_uppercase();

        user.two_factor_code = Some(code.clone());
        user.two_factor_expires_at = Some(
            (chrono::Utc::now() + chrono::Duration::hours(1)).fixed_offset(),
        );
        UserRepository::update(db, user).await?;

        // Send email
        let _ = send_password_reset_email(state, tenant_id, email, &code).await;

        Ok(())
    }

    pub async fn set_password(
        db: &DatabaseConnection,
        tenant_id: &str,
        user_id: &str,
        password: &str,
    ) -> Result<UserResponse, ApiError> {
        let mut user = UserRepository::find_by_id(db, user_id, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Utilisateur introuvable".to_string()))?;

        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|e| ApiError::Internal(format!("Erreur lors du hachage : {}", e)))?;

        user.password_hash = password_hash;
        user.two_factor_code = None;
        user.two_factor_expires_at = None;
        let updated = UserRepository::update(db, user).await?;

        let roles = UserRepository::get_user_roles(db, &updated.id).await?;
        Ok(Self::map_to_response(updated, roles))
    }

    fn map_to_response(m: crate::models::user::Model, roles: Vec<String>) -> UserResponse {
        UserResponse {
            id: m.id,
            tenant_id: m.tenant_id,
            name: m.name,
            email: m.email,
            is_active: m.is_active,
            two_factor_enabled: m.two_factor_enabled,
            roles,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}
