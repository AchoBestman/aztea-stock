use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use rust_decimal::Decimal;

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct CreateSubscriptionPayload {
    pub tenant_id: String,
    pub plan: String, // starter, pro, enterprise
    pub status: String, // trial, active, suspended, cancelled
    pub price_monthly: Decimal,
    pub currency: Option<String>,
    pub expires_at: String, // ISO date string
    pub trial_ends_at: Option<String>,
    pub notes: Option<String>,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct SubscriptionResponse {
    #[schema(example = "a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d")]
    pub id: String,
    #[schema(example = "cf4beab2-e84e-471e-90ec-c5bf6c9c4c22")]
    pub tenant_id: String,
    #[schema(example = "pro")]
    pub plan: String,
    #[schema(example = "active")]
    pub status: String,
    #[schema(example = "49.99")]
    pub price_monthly: Decimal,
    #[schema(example = "XAF")]
    pub currency: String,
    #[schema(example = "2026-05-20T10:00:00Z")]
    pub started_at: String,
    #[schema(example = "2026-06-20T10:00:00Z")]
    pub expires_at: String,
    #[schema(example = "2026-05-27T10:00:00Z")]
    pub trial_ends_at: Option<String>,
    #[schema(nullable)]
    pub cancelled_at: Option<String>,
    #[schema(example = "Abonnement pro mensuel")]
    pub notes: Option<String>,
    #[schema(example = "2026-05-20T10:00:00Z")]
    pub created_at: String,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct PaginatedSubscriptionResponse {
    pub data: Vec<SubscriptionResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}
