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
        let rng = rand::thread_rng();
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

    pub async fn list_tenant_licenses(
        db: &DatabaseConnection,
        tenant_id: &str,
    ) -> Result<Vec<LicenseResponse>, ApiError> {
        let models = license::Entity::find()
            .filter(license::Column::TenantId.eq(tenant_id))
            .all(db)
            .await?;
        
        Ok(models.into_iter().map(Self::map_to_response).collect())
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
}
