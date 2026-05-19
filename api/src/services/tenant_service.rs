use sea_orm::DatabaseConnection;
use crate::{
    errors::ApiError,
    repositories::tenant_repository::TenantRepository,
    dtos::tenant_dto::{CreateTenantPayload, UpdateTenantPayload, TenantResponse},
    utils::crypto::encrypt,
};

pub struct TenantService;

impl TenantService {
    pub async fn create_tenant(
        db: &DatabaseConnection,
        payload: CreateTenantPayload,
    ) -> Result<TenantResponse, ApiError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().fixed_offset();
        let tenant = crate::models::tenant::Model {
            id,
            name: payload.name,
            business_type: payload.business_type,
            email: payload.email,
            phone: payload.phone,
            address: payload.address,
            country: payload.country,
            timezone: payload.timezone,
            logo_url: payload.logo_url,
            is_active: Some(true),
            is_system: false,
            two_factor_enabled: false,
            sender_email: None,
            sender_user: None,
            sender_password: None,
            created_at: now,
            updated_at: now,
        };

        let created = TenantRepository::create(db, tenant).await?;
        Ok(Self::map_to_response(created))
    }

    pub async fn list_tenants(
        db: &DatabaseConnection,
        business_type: Option<String>,
        search: Option<String>,
        is_active: Option<bool>,
        created_after: Option<String>,
        created_before: Option<String>,
    ) -> Result<Vec<TenantResponse>, ApiError> {
        let after_parsed = if let Some(ref a) = created_after {
            Some(chrono::DateTime::parse_from_rfc3339(a)
                .map_err(|e| ApiError::BadRequest(format!("Format de date created_after invalide (doit être RFC3339) : {}", e)))?)
        } else {
            None
        };

        let before_parsed = if let Some(ref b) = created_before {
            Some(chrono::DateTime::parse_from_rfc3339(b)
                .map_err(|e| ApiError::BadRequest(format!("Format de date created_before invalide (doit être RFC3339) : {}", e)))?)
        } else {
            None
        };

        let models = TenantRepository::find_all_filtered(
            db,
            business_type.as_deref(),
            search.as_deref(),
            is_active,
            after_parsed,
            before_parsed,
        ).await?;
        let responses = models.into_iter().map(Self::map_to_response).collect();
        Ok(responses)
    }

    pub async fn get_tenant(
        db: &DatabaseConnection,
        tenant_id: &str,
    ) -> Result<TenantResponse, ApiError> {
        let tenant = Self::load_tenant(db, tenant_id, "Tenant introuvable").await?;

        Ok(Self::map_to_response(tenant))
    }

    pub async fn update_tenant(
        db: &DatabaseConnection,
        caller_tenant_id: &str,
        target_tenant_id: &str,
        payload: UpdateTenantPayload,
        caller_has_credentials_permission: bool,
    ) -> Result<TenantResponse, ApiError> {
        // Load caller tenant
        let caller_tenant = Self::load_tenant(db, caller_tenant_id, "Tenant de l'utilisateur introuvable").await?;

        // 1. Guard: Only system tenant users can update another tenant
        crate::utils::auth::require_tenant_access(db, caller_tenant_id, target_tenant_id).await?;

        // Load target tenant
        let mut tenant = Self::load_tenant(db, target_tenant_id, "Tenant introuvable").await?;

        // 2. Guard: Identify which fields the caller wants to modify
        let system_only_fields_modified = payload.business_type.is_some()
            || payload.country.is_some()
            || payload.is_active.is_some()
            || payload.name.is_some()
            || payload.sender_email.is_some()
            || payload.sender_password.is_some()
            || payload.sender_user.is_some();

        if system_only_fields_modified && !caller_tenant.is_system {
            return Err(ApiError::Unauthorized(
                "Seul un utilisateur du tenant système est autorisé à modifier ces informations.".to_string()
            ));
        }

        // 3. Guard: specific permission needed to modify credentials
        let credentials_fields_modified = payload.sender_email.is_some()
            || payload.sender_password.is_some()
            || payload.sender_user.is_some();

        if credentials_fields_modified && !caller_has_credentials_permission {
            return Err(ApiError::Unauthorized(
                "Vous n'avez pas la permission spécifique pour modifier les informations de connexion SMTP.".to_string()
            ));
        }

        // Apply changes
        if let Some(name) = payload.name {
            tenant.name = name;
        }
        if let Some(bt) = payload.business_type {
            tenant.business_type = bt;
        }
        if let Some(email) = payload.email {
            tenant.email = email;
        }
        if let Some(phone) = payload.phone {
            tenant.phone = phone;
        }
        if let Some(address) = payload.address {
            tenant.address = address;
        }
        if let Some(country) = payload.country {
            tenant.country = country;
        }
        if let Some(timezone) = payload.timezone {
            tenant.timezone = timezone;
        }
        if let Some(logo_url) = payload.logo_url {
            tenant.logo_url = logo_url;
        }
        if let Some(is_active) = payload.is_active {
            tenant.is_active = is_active;
        }
        if let Some(sender_email) = payload.sender_email {
            tenant.sender_email = sender_email;
        }
        if let Some(two_factor) = payload.two_factor_enabled {
            tenant.two_factor_enabled = two_factor;
        }

        // Encryption logic for sender_user and sender_password
        if let Some(sender_user) = payload.sender_user {
            let sender_user_str = sender_user.unwrap_or_default();
            if !sender_user_str.is_empty() {
                tenant.sender_user = Some(encrypt(&sender_user_str));
            } else {
                tenant.sender_user = None;
            }
        }
        if let Some(sender_password) = payload.sender_password {
            let sender_password_str = sender_password.unwrap_or_default();
            if !sender_password_str.is_empty() {
                tenant.sender_password = Some(encrypt(&sender_password_str));
            } else {
                tenant.sender_password = None;
            }
        }

        tenant.updated_at = chrono::Utc::now().fixed_offset();

        let updated = TenantRepository::update(db, tenant).await?;
        Ok(Self::map_to_response(updated))
    }

    pub async fn delete_tenant(
        db: &DatabaseConnection,
        caller_tenant_id: &str,
        target_tenant_id: &str,
    ) -> Result<TenantResponse, ApiError> {
        let caller_tenant = Self::load_tenant(db, caller_tenant_id, "Tenant de l'utilisateur introuvable").await?;

        if !caller_tenant.is_system {
            return Err(ApiError::Unauthorized(
                "Seul un utilisateur du tenant système est autorisé à supprimer un tenant.".to_string()
            ));
        }

        let mut tenant = Self::load_tenant(db, target_tenant_id, "Tenant introuvable").await?;

        tenant.is_active = Some(false);
        tenant.updated_at = chrono::Utc::now().fixed_offset();

        let updated = TenantRepository::update(db, tenant).await?;
        Ok(Self::map_to_response(updated))
    }

    pub async fn set_two_factor(
        db: &DatabaseConnection,
        tenant_id: &str,
        enabled: bool,
    ) -> Result<TenantResponse, ApiError> {
        let mut tenant = Self::load_tenant(db, tenant_id, "Tenant introuvable").await?;

        tenant.two_factor_enabled = enabled;
        tenant.updated_at = chrono::Utc::now().fixed_offset();

        let updated = TenantRepository::update(db, tenant).await?;
        Ok(Self::map_to_response(updated))
    }

    fn map_to_response(m: crate::models::tenant::Model) -> TenantResponse {
        TenantResponse {
            id: m.id,
            name: m.name,
            business_type: m.business_type,
            email: m.email,
            phone: m.phone,
            address: m.address,
            country: m.country,
            timezone: m.timezone,
            logo_url: m.logo_url,
            is_active: m.is_active,
            is_system: m.is_system,
            two_factor_enabled: m.two_factor_enabled,
            sender_email: m.sender_email,
            sender_user_encrypted: m.sender_user,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }

    pub async fn load_tenant(
        db: &DatabaseConnection,
        id: &str,
        err_msg: &str,
    ) -> Result<crate::models::tenant::Model, ApiError> {
        TenantRepository::find_by_id(db, id)
            .await?
            .ok_or_else(|| ApiError::NotFound(err_msg.to_string()))
    }
}
