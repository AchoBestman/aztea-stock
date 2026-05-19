use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct UpdateTenantPayload {
    pub name: Option<String>,
    pub business_type: Option<String>,
    pub email: Option<String>,
    pub phone: Option<Option<String>>,
    pub address: Option<Option<String>>,
    pub country: Option<Option<String>>,
    pub timezone: Option<Option<String>>,
    pub logo_url: Option<Option<String>>,
    pub is_active: Option<Option<bool>>,
    pub sender_email: Option<Option<String>>,
    pub sender_user: Option<Option<String>>,
    pub sender_password: Option<Option<String>>,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct SetTenantTwoFactorPayload {
    pub tenant_id: Option<String>,
    pub two_factor_enabled: bool,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct TenantResponse {
    pub id: String,
    pub name: String,
    pub business_type: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub country: Option<String>,
    pub timezone: Option<String>,
    pub logo_url: Option<String>,
    pub is_active: Option<bool>,
    pub is_system: bool,
    pub two_factor_enabled: bool,
    pub sender_email: Option<String>,
    pub sender_user_encrypted: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
