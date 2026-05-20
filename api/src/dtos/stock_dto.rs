use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateStockItemPayload {
    pub product_id: String,
    pub quantity: Option<f64>,
    pub quantity_reserved: Option<f64>,
    pub low_stock_threshold: Option<f64>,
    #[validate(length(max = 255))]
    pub unit_location: Option<String>,
    #[validate(length(max = 100))]
    pub batch_number: Option<String>,
    pub expiry_date: Option<String>, // ISO date or simple string
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UpdateStockItemPayload {
    pub quantity: Option<f64>,
    pub quantity_reserved: Option<f64>,
    pub low_stock_threshold: Option<f64>,
    #[validate(length(max = 255))]
    pub unit_location: Option<Option<String>>,
    #[validate(length(max = 100))]
    pub batch_number: Option<Option<String>>,
    pub expiry_date: Option<Option<String>>,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct StockItemResponse {
    #[schema(example = "a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d")]
    pub id: String,
    #[schema(example = "cf4beab2-e84e-471e-90ec-c5bf6c9c4c22")]
    pub tenant_id: String,
    #[schema(example = "d8d3e230-67a6-4ec7-88e8-d1f50a8b98fe")]
    pub product_id: String,
    #[schema(example = "Smartphone Pro 256GB")]
    pub product_name: String,
    #[schema(example = 150.5)]
    pub quantity: f64,
    #[schema(example = 2.0)]
    pub quantity_reserved: f64,
    #[schema(example = 10.0)]
    pub low_stock_threshold: f64,
    #[schema(example = "Rayon A, Étagère 3")]
    pub unit_location: Option<String>,
    #[schema(example = "BATCH-2026-05")]
    pub batch_number: Option<String>,
    #[schema(example = "2027-12-31")]
    pub expiry_date: Option<String>,
    #[schema(example = "2026-05-20T10:00:00Z")]
    pub updated_at: String,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct PaginatedStockItemResponse {
    pub data: Vec<StockItemResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateStockMovementPayload {
    pub product_id: String,
    #[validate(custom(function = "validate_movement_type"))]
    pub movement_type: String, // sale, purchase, adjustment, return, loss, initial
    pub quantity_change: f64,
    pub reference_id: Option<String>,
    pub note: Option<String>,
}

fn validate_movement_type(val: &str) -> Result<(), validator::ValidationError> {
    match val {
        "sale" | "purchase" | "adjustment" | "return" | "loss" | "initial" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_movement_type")),
    }
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct StockMovementResponse {
    #[schema(example = "e5f6a1b2-c3d4-7a8b-9c0d-1e2f3a4b5c6d")]
    pub id: String,
    #[schema(example = "cf4beab2-e84e-471e-90ec-c5bf6c9c4c22")]
    pub tenant_id: String,
    #[schema(example = "d8d3e230-67a6-4ec7-88e8-d1f50a8b98fe")]
    pub product_id: String,
    #[schema(example = "Smartphone Pro 256GB")]
    pub product_name: String,
    #[schema(example = "a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d")]
    pub user_id: Option<String>,
    #[schema(example = "Jean Dupont")]
    pub user_name: Option<String>,
    #[schema(example = "purchase")]
    pub movement_type: String,
    #[schema(example = 50.0)]
    pub quantity_before: f64,
    #[schema(example = 20.0)]
    pub quantity_change: f64,
    #[schema(example = 70.0)]
    pub quantity_after: f64,
    #[schema(example = "f5e6d7c8-b9a0-1234-5678-9abcdef01234")]
    pub reference_id: Option<String>,
    #[schema(example = "Approvisionnement mensuel de produits")]
    pub note: Option<String>,
    #[schema(example = "2026-05-20T10:00:00Z")]
    pub occurred_at: String,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct PaginatedStockMovementResponse {
    pub data: Vec<StockMovementResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}
