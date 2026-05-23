use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

// --- Sales ---

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateSaleItemPayload {
    pub product_id: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub tax_rate: Option<f64>,
    pub discount: Option<f64>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateSalePayload {
    pub tenant_id: Option<String>,
    #[validate(length(min = 1))]
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub payment_method: String, // cash, card, mobile_money, credit
    pub notes: Option<String>,
    pub amount_paid: Option<f64>,
    pub change_given: Option<f64>,
    #[validate(nested)]
    pub items: Vec<CreateSaleItemPayload>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct RefundSalePayload {
    pub refund_items: Vec<RefundItemPayload>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct RefundItemPayload {
    pub product_id: String,
    pub quantity: f64,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct SaleItemResponse {
    #[schema(example = "s1i2_uuid")]
    pub id: String,
    #[schema(example = "prod_uuid")]
    pub product_id: String,
    #[schema(example = "Coca Cola 33cl")]
    pub product_name: String,
    #[schema(example = "5449000000096")]
    pub product_barcode: Option<String>,
    #[schema(example = 2.0)]
    pub quantity: f64,
    #[schema(example = 1.5)]
    pub unit_price: f64,
    #[schema(example = 18.0)]
    pub tax_rate: f64,
    #[schema(example = 0.0)]
    pub discount: f64,
    #[schema(example = 3.0)]
    pub line_total: f64,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct SaleResponse {
    #[schema(example = "sale_uuid")]
    pub id: String,
    #[schema(example = "tenant_uuid")]
    pub tenant_id: String,
    #[schema(example = "user_uuid")]
    pub user_id: Option<String>,
    #[schema(example = "FAC-0001")]
    pub receipt_number: String,
    #[schema(example = "Jean Dupont")]
    pub customer_name: Option<String>,
    #[schema(example = "+242 06 999 9999")]
    pub customer_phone: Option<String>,
    #[schema(example = 10.0)]
    pub subtotal: f64,
    #[schema(example = 1.8)]
    pub tax_total: f64,
    #[schema(example = 0.5)]
    pub discount_total: f64,
    #[schema(example = 11.3)]
    pub total: f64,
    #[schema(example = 15.0)]
    pub amount_paid: f64,
    #[schema(example = 3.7)]
    pub change_given: f64,
    #[schema(example = "cash")]
    pub payment_method: String,
    #[schema(example = "completed")]
    pub status: String,
    pub notes: Option<String>,
    pub sold_at: String,
    pub created_at: String,
    pub items: Vec<SaleItemResponse>,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct ReceiptPrintResponse {
    pub receipt_number: String,
    pub date: String,
    pub cashier: Option<String>,
    pub customer_name: Option<String>,
    pub subtotal: f64,
    pub tax_total: f64,
    pub discount_total: f64,
    pub total: f64,
    pub payment_method: String,
    pub lines: Vec<ReceiptItemLine>,
    pub footer_note: Option<String>,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct ReceiptItemLine {
    pub name: String,
    pub qty: f64,
    pub price: f64,
    pub total: f64,
}

// --- Purchases ---

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreatePurchaseItemPayload {
    pub product_id: String,
    pub quantity: f64,
    pub unit_cost: f64,
    pub expiry_date: Option<String>,
    pub batch_number: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreatePurchasePayload {
    pub tenant_id: Option<String>,
    pub supplier_name: Option<String>,
    pub supplier_phone: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    #[validate(nested)]
    pub items: Vec<CreatePurchaseItemPayload>,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct PurchaseItemResponse {
    #[schema(example = "pi_uuid")]
    pub id: String,
    #[schema(example = "prod_uuid")]
    pub product_id: String,
    #[schema(example = "Phone Charger")]
    pub product_name: String,
    #[schema(example = 10.0)]
    pub quantity: f64,
    #[schema(example = 5.0)]
    pub unit_cost: f64,
    #[schema(example = "2027-05-20")]
    pub expiry_date: Option<String>,
    #[schema(example = "LOT-102")]
    pub batch_number: Option<String>,
    #[schema(example = 50.0)]
    pub line_total: f64,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct PurchaseResponse {
    #[schema(example = "purchase_uuid")]
    pub id: String,
    #[schema(example = "tenant_uuid")]
    pub tenant_id: String,
    #[schema(example = "user_uuid")]
    pub user_id: Option<String>,
    #[schema(example = "Fournisseur Alpha")]
    pub supplier_name: Option<String>,
    #[schema(example = "+242 05 555 5555")]
    pub supplier_phone: Option<String>,
    #[schema(example = "REF-2026-003")]
    pub reference: Option<String>,
    #[schema(example = 50.0)]
    pub total: f64,
    #[schema(example = "received")]
    pub status: String,
    pub notes: Option<String>,
    pub purchased_at: String,
    pub created_at: String,
    pub items: Vec<PurchaseItemResponse>,
}

// --- Alerts ---

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct AlertResponse {
    #[schema(example = "alert_uuid")]
    pub id: String,
    #[schema(example = "tenant_uuid")]
    pub tenant_id: String,
    #[schema(example = "prod_uuid")]
    pub product_id: Option<String>,
    #[schema(example = "Smartphone Pro")]
    pub product_name: Option<String>,
    #[schema(example = "low_stock")]
    pub alert_type: String,
    #[schema(example = "Stock de Smartphone Pro inférieur au seuil d'alerte (2 restant).")]
    pub message: String,
    #[schema(example = 5.0)]
    pub threshold: Option<f64>,
    #[schema(example = 2.0)]
    pub current_qty: Option<f64>,
    #[schema(example = false)]
    pub is_read: bool,
    #[schema(example = false)]
    pub is_resolved: bool,
    pub triggered_at: String,
}

// --- Sync Log ---

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateSyncLogPayload {
    pub tenant_id: Option<String>,
    pub device_id: String,
    pub sync_type: String, // push, pull, full
    pub status: String, // success, partial, failed
    pub records_pushed: i32,
    pub records_pulled: i32,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct SyncLogResponse {
    #[schema(example = "sync_uuid")]
    pub id: String,
    #[schema(example = "tenant_uuid")]
    pub tenant_id: String,
    #[schema(example = "device_001")]
    pub device_id: String,
    #[schema(example = "push")]
    pub sync_type: Option<String>,
    #[schema(example = "success")]
    pub status: Option<String>,
    #[schema(example = 12)]
    pub records_pushed: i32,
    #[schema(example = 5)]
    pub records_pulled: i32,
    pub error_message: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct PaginatedSaleResponse {
    pub data: Vec<SaleResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct PaginatedPurchaseResponse {
    pub data: Vec<PurchaseResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct PaginatedAlertResponse {
    pub data: Vec<AlertResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct PaginatedSyncLogResponse {
    pub data: Vec<SyncLogResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}
