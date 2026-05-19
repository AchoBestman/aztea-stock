use sea_orm::DatabaseConnection;
use crate::{
    errors::ApiError,
    repositories::tenant_repository::TenantRepository,
    dtos::tenant_dto::{UpdateTenantPayload, TenantResponse},
    utils::crypto::encrypt,
};

pub struct TenantService;

impl TenantService {
    pub async fn get_tenant(
        db: &DatabaseConnection,
        tenant_id: &str,
    ) -> Result<TenantResponse, ApiError> {
        let tenant = TenantRepository::find_by_id(db, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Tenant introuvable".to_string()))?;

        Ok(Self::map_to_response(tenant))
    }

    pub async fn update_tenant(
        db: &DatabaseConnection,
        tenant_id: &str,
        payload: UpdateTenantPayload,
    ) -> Result<TenantResponse, ApiError> {
        let mut tenant = TenantRepository::find_by_id(db, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Tenant introuvable".to_string()))?;

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

        // Encryption logic for sender_user and sender_password
        if let Some(sender_user) = payload.sender_user {
            let sender_user_str = sender_user.unwrap_or_default();
            if !sender_user_str.is_empty() {
                tenant.sender_user = Some(encrypt(&sender_user_str));
            }
        }
        if let Some(sender_password) = payload.sender_password {
            let sender_password_str = sender_password.unwrap_or_default();
            if !sender_password_str.is_empty() {
                tenant.sender_password = Some(encrypt(&sender_password_str));
            }
        }

        tenant.updated_at = chrono::Utc::now().fixed_offset();

        let updated = TenantRepository::update(db, tenant).await?;
        Ok(Self::map_to_response(updated))
    }

    pub async fn set_two_factor(
        db: &DatabaseConnection,
        tenant_id: &str,
        enabled: bool,
    ) -> Result<TenantResponse, ApiError> {
        let mut tenant = TenantRepository::find_by_id(db, tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Tenant introuvable".to_string()))?;

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
}
