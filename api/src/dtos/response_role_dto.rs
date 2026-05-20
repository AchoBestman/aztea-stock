use serde::Serialize;
use utoipa::ToSchema;
use crate::services::permission_service::PermissionResponse;

#[derive(Serialize, Clone, Debug, ToSchema)]
pub struct RoleResponse {
    #[schema(example = "a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d")]
    pub id: String,
    #[schema(example = "cf4beab2-e84e-471e-90ec-c5bf6c9c4c22")]
    pub tenant_id: String,
    #[schema(example = "Vendeur")]
    pub name: String,
    #[schema(example = "Rôle de vendeur en caisse")]
    pub description: Option<String>,
    pub permissions: Option<Vec<PermissionResponse>>,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct PaginatedRoleResponse {
    pub data: Vec<RoleResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Serialize, ToSchema)]
pub struct DeleteRoleResponse {
    pub success: bool,
    pub message: String,
}
