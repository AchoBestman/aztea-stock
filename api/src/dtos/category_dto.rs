use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateCategoryPayload {
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(max = 7))]
    pub color: Option<String>,
    #[validate(length(max = 100))]
    pub icon: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UpdateCategoryPayload {
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    pub description: Option<String>,
    #[validate(length(max = 7))]
    pub color: Option<String>,
    #[validate(length(max = 100))]
    pub icon: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryResponse {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub parent_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
