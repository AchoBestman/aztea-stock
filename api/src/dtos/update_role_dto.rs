use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct UpdateRolePayload {
    pub name: String,
    pub description: Option<String>,
}
