use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct CreateTenantPayload {
    pub name: String,
    pub business_type: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub country: String,
    /// Code pays ISO (ex: CG, FR)
    pub country_code: Option<String>,
    pub city: String,
    pub timezone: String,
    pub logo_url: Option<String>,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct UpdateTenantPayload {
    pub name: Option<String>,
    pub business_type: Option<String>,
    pub email: Option<String>,
    pub phone: Option<Option<String>>,
    pub address: Option<Option<String>>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub timezone: Option<String>,
    pub logo_url: Option<Option<String>>,
    pub is_active: Option<Option<bool>>,
    pub sender_email: Option<Option<String>>,
    pub sender_user: Option<Option<String>>,
    pub sender_password: Option<Option<String>>,
    pub two_factor_enabled: Option<bool>,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct SetTenantTwoFactorPayload {
    pub tenant_id: Option<String>,
    pub two_factor_enabled: bool,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct TenantResponse {
    #[schema(example = "cf4beab2-e84e-471e-90ec-c5bf6c9c4c22")]
    pub id: String,
    #[schema(example = "Aztea Corp")]
    pub name: String,
    #[schema(example = "supermarket")]
    pub business_type: String,
    #[schema(example = "contact@aztea.com")]
    pub email: String,
    #[schema(example = "+242060000000")]
    pub phone: Option<String>,
    #[schema(example = "12 Avenue de la Paix")]
    pub address: Option<String>,
    #[schema(example = "Brazzaville")]
    pub city: Option<String>,
    pub country: Option<String>,
    #[schema(example = "CG")]
    pub country_code: Option<String>,
    #[schema(example = "Africa/Brazzaville")]
    pub timezone: Option<String>,
    #[schema(example = "https://cdn.aztea.com/logo.png")]
    pub logo_url: Option<String>,
    #[schema(example = true)]
    pub is_active: Option<bool>,
    #[schema(example = false)]
    pub is_system: bool,
    #[schema(example = false)]
    pub two_factor_enabled: bool,
    #[schema(example = "smtp@aztea.com")]
    pub sender_email: Option<String>,
    #[schema(example = "smtp_user_encrypted_hash")]
    pub sender_user_encrypted: Option<String>,
    #[schema(example = "2026-05-20T10:00:00Z")]
    pub created_at: String,
    #[schema(example = "2026-05-20T10:00:00Z")]
    pub updated_at: String,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct PaginatedTenantResponse {
    pub data: Vec<TenantResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}
