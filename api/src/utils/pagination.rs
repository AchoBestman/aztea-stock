use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams, Clone, Debug)]
pub struct PaginationParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub search: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub tenant_id: Option<String>,
    /// Column name to sort by (e.g. "created_at", "name", "status")
    pub order_by: Option<String>,
    /// "asc" or "desc" (default: "desc")
    pub order_type: Option<String>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
            search: None,
            start_date: None,
            end_date: None,
            tenant_id: None,
            order_by: None,
            order_type: None,
        }
    }
}

#[derive(Serialize, ToSchema, Clone, Debug)]
#[aliases(
    PaginatedSubscriptionResponse = PaginatedResponse<crate::dtos::subscription_dto::SubscriptionResponse>,
    PaginatedLicenseResponse = PaginatedResponse<crate::dtos::license_dto::LicenseResponse>,
    PaginatedTenantResponse = PaginatedResponse<crate::dtos::tenant_dto::TenantResponse>,
    PaginatedUserResponse = PaginatedResponse<crate::dtos::user_dto::UserResponse>,
    PaginatedRoleResponse = PaginatedResponse<crate::dtos::response_role_dto::RoleResponse>
)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}
