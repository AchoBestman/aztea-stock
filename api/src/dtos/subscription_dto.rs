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
    pub id: String,
    pub tenant_id: String,
    pub plan: String,
    pub status: String,
    pub price_monthly: Decimal,
    pub currency: String,
    pub started_at: String,
    pub expires_at: String,
    pub trial_ends_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}
