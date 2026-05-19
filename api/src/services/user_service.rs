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
        caller_tenant_id: &str,
        payload: CreateUserPayload,
    ) -> Result<UserResponse, ApiError> {
        let db = state.db.as_ref().ok_or_else(|| {
            ApiError::Internal("Base de données indisponible".to_string())
        })?;

        // Resolve target tenant
        let mut target_tenant_id = caller_tenant_id.to_string();
        if let Some(requested_tenant) = &payload.tenant_id {
            use sea_orm::EntityTrait;
            let caller_tenant = crate::models::tenant::Entity::find_by_id(caller_tenant_id)
                .one(db)
                .await?
                .ok_or_else(|| ApiError::Unauthorized("Tenant introuvable".to_string()))?;

            if caller_tenant.is_system {
                target_tenant_id = requested_tenant.clone();
            } else {
                return Err(ApiError::Forbidden("Seul le tenant système peut spécifier un tenant_id".to_string()));
            }
        }

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
            &target_tenant_id,
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
        let _ = send_password_reset_email(state, &target_tenant_id, &payload.email, &code).await;

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

    pub async fn send_public_password_reset(
        state: &AppState,
        email: &str,
    ) -> Result<(), ApiError> {
        let db = state.db.as_ref().ok_or_else(|| {
            ApiError::Internal("Base de données indisponible".to_string())
        })?;

        let mut user = UserRepository::find_by_email(db, email)
            .await?
            .ok_or_else(|| ApiError::NotFound("Utilisateur introuvable".to_string()))?;

        let code: String = (0..6)
            .map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric) as char)
            .collect::<String>()
            .to_uppercase();

        user.two_factor_code = Some(code.clone());
        user.two_factor_expires_at = Some(
            (chrono::Utc::now() + chrono::Duration::hours(1)).fixed_offset(),
        );
        let tenant_id = user.tenant_id.clone();
        UserRepository::update(db, user).await?;

        let _ = send_password_reset_email(state, &tenant_id, email, &code).await;

        Ok(())
    }

    pub async fn public_reset_password(
        db: &DatabaseConnection,
        email: &str,
        otp_code: &str,
        new_password: &str,
    ) -> Result<(), ApiError> {
        let mut user = UserRepository::find_by_email(db, email)
            .await?
            .ok_or_else(|| ApiError::NotFound("Utilisateur introuvable".to_string()))?;

        if let Some(ref stored_code) = user.two_factor_code {
            if stored_code != otp_code {
                return Err(ApiError::BadRequest("Code de validation incorrect".to_string()));
            }
        } else {
            return Err(ApiError::BadRequest("Aucun code de validation n'a été demandé".to_string()));
        }

        if let Some(expires_at) = user.two_factor_expires_at {
            if chrono::Utc::now().fixed_offset() > expires_at {
                return Err(ApiError::BadRequest("Le code de validation a expiré".to_string()));
            }
        } else {
            return Err(ApiError::BadRequest("Le code de validation a expiré".to_string()));
        }

        let password_hash = hash(new_password, DEFAULT_COST)
            .map_err(|e| ApiError::Internal(format!("Erreur lors du hachage : {}", e)))?;

        user.password_hash = password_hash;
        user.two_factor_code = None;
        user.two_factor_expires_at = None;
        UserRepository::update(db, user).await?;

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

    pub async fn get_profile(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<crate::dtos::user_dto::UserProfileResponse, ApiError> {
        use sea_orm::EntityTrait;
        use crate::models::{user, tenant};

        // 1. Fetch user
        let user = user::Entity::find_by_id(user_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| ApiError::NotFound("Utilisateur introuvable".to_string()))?;

        // 2. Fetch tenant
        let tenant = tenant::Entity::find_by_id(user.tenant_id.clone())
            .one(db)
            .await?
            .ok_or_else(|| ApiError::NotFound("Tenant introuvable".to_string()))?;

        Ok(crate::dtos::user_dto::UserProfileResponse {
            id: user.id,
            name: user.name,
            email: user.email,
            is_active: user.is_active,
            two_factor_enabled: user.two_factor_enabled,
            tenant: crate::dtos::user_dto::UserProfileTenantResponse {
                name: tenant.name,
                email: tenant.email,
                phone: tenant.phone,
                country: tenant.country,
                address: tenant.address,
                business_type: tenant.business_type,
                created_at: tenant.created_at.to_rfc3339(),
                is_active: tenant.is_active,
            },
        })
    }

    pub async fn update_profile(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        payload: crate::dtos::user_dto::UpdateProfilePayload,
    ) -> Result<crate::dtos::user_dto::UserProfileResponse, ApiError> {
        use sea_orm::{EntityTrait, ActiveModelTrait, Set};
        use crate::models::user;
        use crate::utils::auth::check_permission;

        // 1. Determine target user_id (if not provided, it's the caller themselves)
        let target_user_id = payload.user_id.clone().unwrap_or_else(|| caller_user_id.to_string());
        let is_self = target_user_id == caller_user_id;

        // 2. Fetch target user
        let target_user = user::Entity::find_by_id(target_user_id.clone())
            .one(db)
            .await?
            .ok_or_else(|| ApiError::NotFound("Utilisateur introuvable".to_string()))?;

        // 3. System Super Admin check
        let is_system_sa = crate::services::role_service::RoleService::is_system_super_admin(db, caller_user_id, caller_tenant_id).await.unwrap_or(false);

        if !is_system_sa {
            // Multi-tenant check: regular users can only modify users in their own tenant
            if target_user.tenant_id != caller_tenant_id {
                return Err(ApiError::Unauthorized(
                    "Vous n'êtes pas autorisé à modifier les données d'un utilisateur d'un autre tenant.".to_string()
                ));
            }

            // A. If changing name
            if payload.name.is_some() {
                if !is_self {
                    return Err(ApiError::Forbidden(
                        "Vous ne pouvez pas modifier le nom d'un autre utilisateur.".to_string()
                    ));
                }
            }

            // B. If changing is_active (status)
            if payload.is_active.is_some() {
                let has_status_perm = check_permission(db, caller_user_id, "can_update_user_status").await.unwrap_or(false);
                if !has_status_perm {
                    return Err(ApiError::Forbidden(
                        "Vous n'avez pas la permission de modifier le statut d'un utilisateur.".to_string()
                    ));
                }
            }

            // C. If changing two_factor_enabled
            if payload.two_factor_enabled.is_some() {
                let has_2fa_perm = check_permission(db, caller_user_id, "can_update_user_two_factor").await.unwrap_or(false);
                if !has_2fa_perm {
                    return Err(ApiError::Forbidden(
                        "Vous n'avez pas la permission de modifier la double authentification d'un utilisateur.".to_string()
                    ));
                }
            }
        }

        // 4. Apply changes
        let mut active_user: user::ActiveModel = target_user.into();

        if let Some(name) = payload.name {
            active_user.name = Set(name);
        }
        if let Some(is_active) = payload.is_active {
            active_user.is_active = Set(Some(is_active));
        }
        if let Some(two_factor_enabled) = payload.two_factor_enabled {
            active_user.two_factor_enabled = Set(two_factor_enabled);
        }

        let updated_user = active_user.update(db).await?;

        // 5. Return updated profile
        Self::get_profile(db, &updated_user.id).await
    }
}
