use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct CreateUserPayload {
    pub name: String,
    pub email: String,
    pub role_id: String,
    pub tenant_id: Option<String>,
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
    #[schema(example = "a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d")]
    pub id: String,
    #[schema(example = "cf4beab2-e84e-471e-90ec-c5bf6c9c4c22")]
    pub tenant_id: String,
    #[schema(example = "Jean Dupont")]
    pub name: String,
    #[schema(example = "jean.dupont@example.com")]
    pub email: String,
    #[schema(example = true)]
    pub is_active: Option<bool>,
    #[schema(example = false)]
    pub two_factor_enabled: bool,
    #[schema(example = json!(["Vendeur", "Gestionnaire"]))]
    pub roles: Vec<String>,
    #[schema(example = "2026-05-20T10:00:00Z")]
    pub created_at: String,
    #[schema(example = "2026-05-20T10:00:00Z")]
    pub updated_at: String,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct PaginatedUserResponse {
    pub data: Vec<UserResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct UserProfileTenantResponse {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub country: Option<String>,
    pub address: Option<String>,
    pub business_type: String,
    pub logo_url: Option<String>,
    pub created_at: String,
    pub is_active: Option<bool>,
}

impl UserProfileTenantResponse {
    pub fn from_tenant(tenant: &crate::models::tenant::Model) -> Self {
        Self {
            name: tenant.name.clone(),
            email: tenant.email.clone(),
            phone: tenant.phone.clone(),
            country: tenant.country.clone(),
            address: tenant.address.clone(),
            business_type: tenant.business_type.clone(),
            logo_url: tenant.logo_url.clone(),
            created_at: tenant.created_at.to_rfc3339(),
            is_active: tenant.is_active,
        }
    }
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct UserProfileResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub is_active: Option<bool>,
    pub two_factor_enabled: bool,
    pub tenant: UserProfileTenantResponse,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct UpdateProfilePayload {
    pub user_id: Option<String>,
    pub name: Option<String>,
    pub is_active: Option<bool>,
    pub two_factor_enabled: Option<bool>,
}
