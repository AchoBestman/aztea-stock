/// Internal endpoint called by the Cloudflare Worker (or any trusted caller)
/// to actually send an email.
///
/// Protected by `x-internal-secret` header that must match `CRON_SECRET` env var.
/// This mirrors the `/api/internal/send-email` route in aztea-store.
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::services::email_service::{send_email_direct, EmailJob};
use crate::AppState;

#[derive(Deserialize)]
pub struct InternalSendEmailPayload {
    #[serde(flatten)]
    pub job: EmailJob,
}

pub async fn send_email_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<EmailJob>,
) -> StatusCode {
    // ── Validate internal secret ──────────────────────────────────────────────
    let cron_secret = std::env::var("CRON_SECRET").unwrap_or_default();
    let provided = headers
        .get("x-internal-secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if cron_secret.is_empty() || provided != cron_secret {
        tracing::warn!("❌ /internal/send-email: unauthorized attempt");
        return StatusCode::UNAUTHORIZED;
    }

    // ── Send directly via SMTP (tenant credentials resolved inside) ───────────
    match send_email_direct(&state, &payload.tenant_id, &payload.to, &payload.subject, &payload.html).await {
        Ok(_) => {
            tracing::info!("✅ [CF Worker→Internal] Email sent to {}", payload.to);
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("❌ [CF Worker→Internal] Failed to send email to {}: {}", payload.to, e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/send-email", post(send_email_handler))
}
