use axum::{
    routing::{get, post},
    Router,
};
use crate::controllers::gescom_controller::{
    create_sale, list_sales, get_sale, void_sale, refund_sale, get_sale_receipt, export_sales,
    create_purchase, list_purchases, get_purchase, cancel_purchase,
    list_alerts, mark_alert_read, mark_all_alerts_read,
    create_sync_log, list_sync_logs,
};
use std::sync::Arc;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Sales
        .route("/sales", post(create_sale))
        .route("/sales", get(list_sales))
        .route("/sales/export", get(export_sales))
        .route("/sales/:id", get(get_sale))
        .route("/sales/:id/void", post(void_sale))
        .route("/sales/:id/refund", post(refund_sale))
        .route("/sales/:id/receipt", get(get_sale_receipt))
        // Purchases
        .route("/purchases", post(create_purchase))
        .route("/purchases", get(list_purchases))
        .route("/purchases/:id", get(get_purchase))
        .route("/purchases/:id/cancel", post(cancel_purchase))
        // Alerts
        .route("/alerts", get(list_alerts))
        .route("/alerts/:id/read", post(mark_alert_read))
        .route("/alerts/read-all", post(mark_all_alerts_read))
        // Sync Logs
        .route("/sync/logs", post(create_sync_log))
        .route("/sync/logs", get(list_sync_logs))
}
