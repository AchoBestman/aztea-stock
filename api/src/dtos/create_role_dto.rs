use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateRolePayload {
    pub name: String,
    pub description: Option<String>,
}
