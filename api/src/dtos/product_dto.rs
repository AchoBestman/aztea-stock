use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateProductPayload {
    pub category_id: Option<String>,
    #[validate(length(max = 100))]
    pub barcode: Option<String>,
    #[validate(length(min = 2, max = 500))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(max = 255))]
    pub brand: Option<String>,
    #[validate(length(max = 50))]
    pub unit: Option<String>,
    pub purchase_price: Option<f64>,
    pub selling_price: f64,
    pub tax_rate: Option<f64>,
    pub image_url: Option<String>,
    pub is_active: Option<bool>,
    pub requires_prescription: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UpdateProductPayload {
    pub category_id: Option<Option<String>>,
    #[validate(length(max = 100))]
    pub barcode: Option<Option<String>>,
    #[validate(length(min = 2, max = 500))]
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    #[validate(length(max = 255))]
    pub brand: Option<Option<String>>,
    #[validate(length(max = 50))]
    pub unit: Option<String>,
    pub purchase_price: Option<f64>,
    pub selling_price: Option<f64>,
    pub tax_rate: Option<f64>,
    pub image_url: Option<Option<String>>,
    pub is_active: Option<bool>,
    pub requires_prescription: Option<bool>,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct ProductResponse {
    #[schema(example = "a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d")]
    pub id: String,
    #[schema(example = "cf4beab2-e84e-471e-90ec-c5bf6c9c4c22")]
    pub tenant_id: String,
    #[schema(example = "d8d3e230-67a6-4ec7-88e8-d1f50a8b98fe")]
    pub category_id: Option<String>,
    #[schema(example = "Électronique")]
    pub category_name: Option<String>,
    #[schema(example = "6901234567890")]
    pub barcode: Option<String>,
    #[schema(example = "Smartphone Pro 256GB")]
    pub name: String,
    #[schema(example = "Écran OLED, 8GB RAM, triple caméra")]
    pub description: Option<String>,
    #[schema(example = "Samsung")]
    pub brand: Option<String>,
    #[schema(example = "unité")]
    pub unit: String,
    #[schema(example = 550.0)]
    pub purchase_price: f64,
    #[schema(example = 799.0)]
    pub selling_price: f64,
    #[schema(example = 18.0)]
    pub tax_rate: f64,
    #[schema(example = "https://images.example.com/smartphone.png")]
    pub image_url: Option<String>,
    #[schema(example = true)]
    pub is_active: bool,
    #[schema(example = false)]
    pub requires_prescription: bool,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct PaginatedProductResponse {
    pub data: Vec<ProductResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}
