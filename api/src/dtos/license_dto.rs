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
    #[schema(example = "a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d")]
    pub id: String,
    #[schema(example = "cf4beab2-e84e-471e-90ec-c5bf6c9c4c22")]
    pub tenant_id: String,
    #[schema(example = "d8d3e230-67a6-4ec7-88e8-d1f50a8b98fe")]
    pub subscription_id: String,
    #[schema(example = "XXXX-XXXX-XXXX-1234")]
    pub license_key_masked: String,
    #[schema(example = true)]
    pub is_active: bool,
    #[schema(example = "Caisse Principale")]
    pub device_name: Option<String>,
    #[schema(example = "8f9a2b3c-4d5e-6f7a-8b9c-0d1e2f3a4b5c")]
    pub device_fingerprint: Option<String>,
    #[schema(example = "2026-05-20T10:00:00Z")]
    pub last_verified_at: Option<String>,
    #[schema(example = "2026-05-20T10:00:00Z")]
    pub activated_at: Option<String>,
    #[schema(nullable)]
    pub revoked_at: Option<String>,
    #[schema(example = "2026-05-20T10:00:00Z")]
    pub created_at: String,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct PaginatedLicenseResponse {
    pub data: Vec<LicenseResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
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
pub struct RevealLicenseResponse {
    pub id: String,
    pub tenant_id: String,
    pub license_key_plain: String,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct LicenseStatusResponse {
    pub has_active_license: bool,
    pub status: String,
    pub license_id: Option<String>,
    pub subscription_plan: Option<String>,
    pub expires_at: Option<String>,
    pub days_remaining: Option<i64>,
    pub renewal_alert: bool,   // true if ≤ 7 days remaining
}
