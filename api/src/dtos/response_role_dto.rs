use serde::Serialize;
use utoipa::ToSchema;
use crate::services::permission_service::PermissionResponse;

#[derive(Serialize, ToSchema)]
pub struct RoleResponse {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Option<Vec<PermissionResponse>>,
}

#[derive(Serialize, ToSchema)]
pub struct DeleteRoleResponse {
    pub success: bool,
    pub message: String,
}
