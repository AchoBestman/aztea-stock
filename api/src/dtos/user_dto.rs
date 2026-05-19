use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct CreateUserPayload {
    pub name: String,
    pub email: String,
    pub role_id: String,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct SetUserTwoFactorPayload {
    pub user_id: String,
    pub two_factor_enabled: bool,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct SetUserPasswordPayload {
    pub user_id: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct SendPasswordResetPayload {
    pub email: String,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct UserResponse {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub email: String,
    pub is_active: Option<bool>,
    pub two_factor_enabled: bool,
    pub roles: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct UserProfileTenantResponse {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub country: Option<String>,
    pub address: Option<String>,
    pub business_type: String,
    pub created_at: String,
    pub is_active: Option<bool>,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct UserProfileResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub is_active: Option<bool>,
    pub two_factor_enabled: bool,
    pub tenant: UserProfileTenantResponse,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct UpdateProfilePayload {
    pub user_id: Option<String>,
    pub name: Option<String>,
    pub is_active: Option<bool>,
    pub two_factor_enabled: Option<bool>,
}
