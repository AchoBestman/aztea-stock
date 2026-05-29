use crate::{
    dtos::tenant_dto::{CreateTenantPayload, TenantResponse, UpdateTenantPayload},
    errors::ApiError,
    repositories::tenant_repository::TenantRepository,
    utils::crypto::encrypt,
};
use sea_orm::DatabaseConnection;

pub struct TenantService;

impl TenantService {
    fn validate_geo_fields(country: &str, city: &str, timezone: &str) -> Result<(), ApiError> {
        if country.trim().is_empty() {
            return Err(ApiError::BadRequest("Le pays est obligatoire.".to_string()));
        }
        if city.trim().is_empty() {
            return Err(ApiError::BadRequest(
                "La ville est obligatoire.".to_string(),
            ));
        }
        if timezone.trim().is_empty() {
            return Err(ApiError::BadRequest(
                "Le fuseau horaire est obligatoire.".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn create_tenant(
        db: &DatabaseConnection,
        payload: CreateTenantPayload,
    ) -> Result<TenantResponse, ApiError> {
        Self::validate_geo_fields(&payload.country, &payload.city, &payload.timezone)?;

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().fixed_offset();
        let tenant = crate::models::tenant::Model {
            id,
            name: payload.name,
            business_type: payload.business_type,
            email: payload.email,
            phone: payload.phone,
            address: payload.address,
            city: Some(payload.city),
            country: Some(payload.country),
            country_code: payload.country_code,
            timezone: Some(payload.timezone),
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
        is_active: Option<String>,
        created_after: Option<String>,
        created_before: Option<String>,
        country_code: Option<String>,
        page: u64,
        per_page: u64,
        order_by: Option<String>,
        order_type: Option<String>,
    ) -> Result<crate::dtos::tenant_dto::PaginatedTenantResponse, ApiError> {
        let is_active_parsed = if let Some(ref status_str) = is_active {
            Some(Self::parse_bool_param(status_str)?)
        } else {
            None
        };

        let after_parsed = if let Some(ref a) = created_after {
            Some(Self::parse_date_param(a, "created_after")?)
        } else {
            None
        };

        let before_parsed = if let Some(ref b) = created_before {
            Some(Self::parse_date_param(b, "created_before")?)
        } else {
            None
        };

        let order_col = order_by.as_deref().unwrap_or("created_at");
        let order_desc = order_type
            .as_deref()
            .unwrap_or("desc")
            .eq_ignore_ascii_case("desc");

        let (models, total) = TenantRepository::find_paginated(
            db,
            business_type.as_deref(),
            search.as_deref(),
            is_active_parsed,
            after_parsed,
            before_parsed,
            country_code.as_deref(),
            page.max(1),
            per_page.clamp(1, 100),
            order_col,
            order_desc,
        )
        .await?;

        let per_page = per_page.clamp(1, 100);
        let total_pages = if total == 0 {
            0
        } else {
            (total + per_page - 1) / per_page
        };

        let data = models.into_iter().map(Self::map_to_response).collect();

        Ok(crate::dtos::tenant_dto::PaginatedTenantResponse {
            data,
            total,
            page: page.max(1),
            per_page,
            total_pages,
        })
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
        caller_user_id: &str,
        caller_tenant_id: &str,
        target_tenant_id: &str,

        payload: UpdateTenantPayload,
        caller_has_credentials_permission: bool,
    ) -> Result<TenantResponse, ApiError> {
        // Load caller tenant
        let caller_tenant =
            Self::load_tenant(db, caller_tenant_id, "Tenant de l'utilisateur introuvable").await?;

        // 1. Guard: Only system tenant users can update another tenant
        crate::utils::auth::require_tenant_access(
            db,
            caller_tenant_id,
            target_tenant_id,
            caller_user_id,
            "update",
        )
        .await?;

        // Load target tenant
        let mut tenant = Self::load_tenant(db, target_tenant_id, "Tenant introuvable").await?;

        // 2. Guard: Identify which fields the caller wants to modify
        let system_only_fields_modified = payload.business_type.is_some()
            || payload.country.is_some()
            || payload.city.is_some()
            || payload.timezone.is_some()
            || payload.is_active.is_some()
            || payload.name.is_some()
            || payload.sender_email.is_some()
            || payload.sender_password.is_some()
            || payload.sender_user.is_some();

        if system_only_fields_modified && !caller_tenant.is_system {
            return Err(ApiError::Unauthorized(
                "Seul un utilisateur du tenant système est autorisé à modifier ces informations."
                    .to_string(),
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

        if let Some(country) = &payload.country {
            tenant.country = Some(country.clone());
        }
        if let Some(country_code) = &payload.country_code {
            tenant.country_code = Some(country_code.clone());
        }
        if let Some(city) = &payload.city {
            tenant.city = Some(city.clone());
        }
        if let Some(timezone) = &payload.timezone {
            tenant.timezone = Some(timezone.clone());
        }
        if payload.country.is_some()
            || payload.country_code.is_some()
            || payload.city.is_some()
            || payload.timezone.is_some()
        {
            let country = tenant.country.as_deref().unwrap_or("");
            let city = tenant.city.as_deref().unwrap_or("");
            let timezone = tenant.timezone.as_deref().unwrap_or("");
            Self::validate_geo_fields(country, city, timezone)?;
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
        caller_user_id: &str,
        caller_tenant_id: &str,
        target_tenant_id: &str,
    ) -> Result<TenantResponse, ApiError> {
        Self::load_tenant(db, caller_tenant_id, "Tenant de l'utilisateur introuvable").await?;

        crate::utils::auth::require_tenant_access(
            db,
            caller_tenant_id,
            target_tenant_id,
            caller_user_id,
            "delete",
        )
        .await?;

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
            city: m.city,
            country: m.country,
            country_code: m.country_code,
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

    fn parse_bool_param(s: &str) -> Result<bool, ApiError> {
        match s.trim().to_lowercase().as_str() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err(ApiError::BadRequest(format!(
                "Valeur de statut invalide '{}'. Attendu : true, false, 1 ou 0.",
                s
            ))),
        }
    }

    fn parse_date_param(
        s: &str,
        field_name: &str,
    ) -> Result<chrono::DateTime<chrono::FixedOffset>, ApiError> {
        // Try RFC3339 first
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
            return Ok(dt);
        }

        // Try ISO date YYYY-MM-DD
        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            if let Some(naive_datetime) = naive_date.and_hms_opt(0, 0, 0) {
                let offset = chrono::FixedOffset::east_opt(0).unwrap();
                let dt = chrono::DateTime::<chrono::FixedOffset>::from_naive_utc_and_offset(
                    naive_datetime,
                    offset,
                );
                return Ok(dt);
            }
        }

        Err(ApiError::BadRequest(format!(
            "Format de date pour '{}' invalide. Attendu : RFC3339 (ex: 2026-05-19T10:00:00Z) ou ISO date (ex: 2026-05-19).",
            field_name
        )))
    }
}
