use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct GenerateLicensePayload {
    pub tenant_id: String,
    pub subscription_id: String,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct ActivateLicensePayload {
    pub license_key: String,
    pub device_name: Option<String>,
    pub device_fingerprint: Option<String>,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct LicenseResponse {
    pub id: String,

    pub tenant_id: String,
    pub subscription_id: String,
    pub license_key_masked: String,
    pub is_active: bool,
    pub device_name: Option<String>,
    pub device_fingerprint: Option<String>,
    pub last_verified_at: Option<String>,
    pub activated_at: Option<String>,
    pub revoked_at: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct FullLicenseResponse {
    pub id: String,
    pub tenant_id: String,
    pub subscription_id: String,
    pub license_key_plain: String, // Only returned immediately after generation
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct LicenseStatusResponse {
    pub has_active_license: bool,
    pub license_id: Option<String>,
    pub subscription_plan: Option<String>,
    pub expires_at: Option<String>,
    pub days_remaining: Option<i64>,
    pub renewal_alert: bool,   // true if ≤ 7 days remaining
}
