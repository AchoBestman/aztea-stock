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
    pub is_active: Option<bool>,
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
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct CategoryResponse {
    #[schema(example = "d8d3e230-67a6-4ec7-88e8-d1f50a8b98fe")]
    pub id: String,
    #[schema(example = "cf4beab2-e84e-471e-90ec-c5bf6c9c4c22")]
    pub tenant_id: String,
    #[schema(example = "Électronique")]
    pub name: String,
    #[schema(example = "Appareils et gadgets électroniques")]
    pub description: Option<String>,
    #[schema(example = "#4A90E2")]
    pub color: Option<String>,
    #[schema(example = "cpu")]
    pub icon: Option<String>,
    #[schema(example = "a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d")]
    pub parent_id: Option<String>,
    #[schema(example = "Technologie")]
    pub parent_name: Option<String>,
    #[schema(example = true)]
    pub is_active: bool,
    #[schema(example = "2026-05-20T10:00:00Z")]
    pub created_at: String,
    #[schema(example = "2026-05-20T10:00:00Z")]
    pub updated_at: String,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct PaginatedCategoryResponse {
    pub data: Vec<CategoryResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}
