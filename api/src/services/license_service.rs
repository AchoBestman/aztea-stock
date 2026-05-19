use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set};
use rand::Rng;
use crate::{
    errors::ApiError,
    models::{license, subscription},
    dtos::license_dto::{ActivateLicensePayload, FullLicenseResponse, GenerateLicensePayload, LicenseResponse},
    utils::crypto::{encrypt, decrypt},
};

pub struct LicenseService;

impl LicenseService {
    pub fn map_to_response(model: license::Model) -> LicenseResponse {
        LicenseResponse {
            id: model.id,
            tenant_id: model.tenant_id,
            subscription_id: model.subscription_id,
            license_key_masked: "***-***-***".to_string(), // Never return plain text here
            is_active: model.is_active.unwrap_or(false),
            device_name: model.device_name,
            device_fingerprint: model.device_fingerprint,
            last_verified_at: model.last_verified_at.map(|d| d.to_rfc3339()),
            activated_at: model.activated_at.map(|d| d.to_rfc3339()),
            revoked_at: model.revoked_at.map(|d| d.to_rfc3339()),
            created_at: model.created_at.to_rfc3339(),
        }
    }

    fn generate_random_key() -> String {
        let parts: Vec<String> = (0..4)
            .map(|_| {
                let p: String = rand::thread_rng()
                    .sample_iter(&rand::distributions::Alphanumeric)
                    .take(5)
                    .map(char::from)
                    .collect();
                p.to_uppercase()
            })
            .collect();
        parts.join("-")
    }

    pub async fn generate_license(
        db: &DatabaseConnection,
        payload: GenerateLicensePayload,
    ) -> Result<FullLicenseResponse, ApiError> {
        // Verify subscription exists and belongs to the tenant
        let sub = subscription::Entity::find_by_id(&payload.subscription_id)
            .one(db)
            .await?
            .ok_or_else(|| ApiError::NotFound("Abonnement introuvable".to_string()))?;
        
        if sub.tenant_id != payload.tenant_id {
            return Err(ApiError::BadRequest("L'abonnement n'appartient pas à ce tenant".to_string()));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let plain_key = Self::generate_random_key();
        let encrypted_key = encrypt(&plain_key);

        let lic = license::ActiveModel {
            id: Set(id.clone()),
            tenant_id: Set(payload.tenant_id.clone()),
            subscription_id: Set(payload.subscription_id.clone()),
            license_key: Set(encrypted_key),
            is_active: Set(Some(true)),
            created_at: Set(chrono::Utc::now().fixed_offset()),
            ..Default::default()
        };

        let model = lic.insert(db).await.map_err(|e| {
            ApiError::Database(sea_orm::DbErr::Custom(format!("Erreur création licence: {}", e)))
        })?;

        Ok(FullLicenseResponse {
            id: model.id,
            tenant_id: model.tenant_id,
            subscription_id: model.subscription_id,
            license_key_plain: plain_key,
            is_active: true,
            created_at: model.created_at.to_rfc3339(),
        })
    }

    pub async fn list_licenses(
        db: &DatabaseConnection,
        params: crate::utils::pagination::PaginationParams,
        enforce_tenant_id: Option<String>,
    ) -> Result<crate::utils::pagination::PaginatedResponse<LicenseResponse>, ApiError> {
        let mut query = license::Entity::find();

        let target_tenant = enforce_tenant_id.or(params.tenant_id);
        if let Some(tenant_id) = target_tenant {
            query = query.filter(license::Column::TenantId.eq(tenant_id));
        }

        if let Some(search) = params.search {
            query = query.filter(
                sea_orm::Condition::any()
                    .add(license::Column::DeviceName.contains(&search))
            );
        }

        if let Some(start_date) = params.start_date {
            if let Ok(date) = chrono::DateTime::parse_from_rfc3339(&start_date) {
                query = query.filter(license::Column::CreatedAt.gte(date));
            }
        }

        if let Some(end_date) = params.end_date {
            if let Ok(date) = chrono::DateTime::parse_from_rfc3339(&end_date) {
                query = query.filter(license::Column::CreatedAt.lte(date));
            }
        }

        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20);

        use sea_orm::{PaginatorTrait, QueryOrder};
        
        query = query.order_by_desc(license::Column::CreatedAt);

        let paginator = query.paginate(db, per_page);
        let total = paginator.num_items().await?;
        let total_pages = paginator.num_pages().await?;
        
        let models = paginator.fetch_page(page - 1).await?;
        
        Ok(crate::utils::pagination::PaginatedResponse {
            data: models.into_iter().map(Self::map_to_response).collect(),
            total,
            page,
            per_page,
            total_pages,
        })
    }

    pub async fn activate_license(
        db: &DatabaseConnection,
        tenant_id: &str,
        payload: ActivateLicensePayload,
    ) -> Result<LicenseResponse, ApiError> {
        let licenses = license::Entity::find()
            .filter(license::Column::TenantId.eq(tenant_id))
            .all(db)
            .await?;

        for model in licenses {
            let decrypted = decrypt(&model.license_key);
            if decrypted == payload.license_key {
                if model.is_active == Some(false) || model.revoked_at.is_some() {
                    return Err(ApiError::BadRequest("Cette clé de licence a été révoquée ou est inactive".to_string()));
                }
                
                if model.activated_at.is_some() && model.device_fingerprint != payload.device_fingerprint {
                    return Err(ApiError::BadRequest("Cette clé de licence est déjà utilisée par un autre appareil".to_string()));
                }

                let mut active_lic: license::ActiveModel = model.into();
                active_lic.device_name = Set(payload.device_name.clone());
                active_lic.device_fingerprint = Set(payload.device_fingerprint.clone());
                active_lic.activated_at = Set(Some(chrono::Utc::now().fixed_offset()));
                active_lic.last_verified_at = Set(Some(chrono::Utc::now().fixed_offset()));
                
                let updated = active_lic.update(db).await?;
                return Ok(Self::map_to_response(updated));
            }
        }

        Err(ApiError::NotFound("Clé de licence invalide ou introuvable pour cette entreprise".to_string()))
    }

    /// Returns the active license status for a tenant, including subscription plan & expiry.
    pub async fn get_license_status(
        db: &DatabaseConnection,
        tenant_id: &str,
    ) -> Result<crate::dtos::license_dto::LicenseStatusResponse, ApiError> {
        use sea_orm::QueryOrder;

        let license = license::Entity::find()
            .filter(license::Column::TenantId.eq(tenant_id))
            .filter(license::Column::IsActive.eq(true))
            .filter(license::Column::RevokedAt.is_null())
            .order_by_desc(license::Column::CreatedAt)
            .one(db)
            .await?;

        match license {
            None => Ok(crate::dtos::license_dto::LicenseStatusResponse {
                has_active_license: false,
                license_id: None,
                subscription_plan: None,
                expires_at: None,
                days_remaining: None,
                renewal_alert: false,
            }),
            Some(lic) => {
                let sub = subscription::Entity::find_by_id(&lic.subscription_id)
                    .one(db)
                    .await?;

                let (plan, expires_at, days_remaining, renewal_alert) = match sub {
                    None => (None, None, None, false),
                    Some(s) => {
                        let expires = s.expires_at;
                        let now = chrono::Utc::now();
                        let days = (expires.with_timezone(&chrono::Utc) - now).num_days();
                        let alert = days <= 7;
                        (
                            Some(s.plan.clone()),
                            Some(expires.to_rfc3339()),
                            Some(days),
                            alert,
                        )
                    }
                };

                Ok(crate::dtos::license_dto::LicenseStatusResponse {
                    has_active_license: true,
                    license_id: Some(lic.id),
                    subscription_plan: plan,
                    expires_at,
                    days_remaining,
                    renewal_alert,
                })
            }
        }
    }

    /// Background job: checks all active subscriptions. Suspends expired ones and
    /// notifies tenants whose licenses expire within 7 days.
    pub async fn check_and_notify_expiring_licenses(
        state: &crate::AppState,
    ) {
        let db = match state.db.as_ref() {
            Some(db) => db,
            None => return,
        };

        let now = chrono::Utc::now().fixed_offset();
        let alert_threshold = (chrono::Utc::now() + chrono::Duration::days(7)).fixed_offset();

        // 1. Suspend all expired active subscriptions
        let expired = subscription::Entity::find()
            .filter(subscription::Column::Status.eq("active"))
            .filter(subscription::Column::ExpiresAt.lt(now))
            .all(db)
            .await
            .unwrap_or_default();

        for sub in expired {
            tracing::info!("[LicenseTask] Suspending expired subscription {} for tenant {}", sub.id, sub.tenant_id);
            let mut active_sub: subscription::ActiveModel = sub.clone().into();
            active_sub.status = sea_orm::Set("suspended".to_string());
            let _ = sea_orm::ActiveModelTrait::update(active_sub, db).await;

            // Also mark all licenses for this subscription inactive
            let lics = license::Entity::find()
                .filter(license::Column::SubscriptionId.eq(&sub.id))
                .all(db)
                .await
                .unwrap_or_default();

            for lic in lics {
                let mut active_lic: license::ActiveModel = lic.into();
                active_lic.is_active = sea_orm::Set(Some(false));
                let _ = sea_orm::ActiveModelTrait::update(active_lic, db).await;
            }
        }

        // 2. Notify tenants whose subscriptions expire within 7 days
        let expiring_soon = subscription::Entity::find()
            .filter(subscription::Column::Status.eq("active"))
            .filter(subscription::Column::ExpiresAt.gte(now))
            .filter(subscription::Column::ExpiresAt.lte(alert_threshold))
            .all(db)
            .await
            .unwrap_or_default();

        for sub in expiring_soon {
            let days_left = (sub.expires_at.with_timezone(&chrono::Utc) - chrono::Utc::now()).num_days();
            tracing::info!(
                "[LicenseTask] Sending renewal alert for tenant {} — {} days left",
                sub.tenant_id, days_left
            );

            // Fetch tenant email
            let tenant_opt = crate::models::tenant::Entity::find_by_id(&sub.tenant_id)
                .one(db)
                .await
                .unwrap_or(None);

            if let Some(t) = tenant_opt {
                let _ = crate::services::email_service::send_license_renewal_alert(
                    state,
                    &sub.tenant_id,
                    &t.email,
                    &sub.plan,
                    days_left,
                    &sub.expires_at.to_rfc3339(),
                )
                .await;
            }
        }
    }
}
