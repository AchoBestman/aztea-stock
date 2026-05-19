use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, QueryOrder};
use std::sync::Arc;
use crate::AppState;
use crate::models::{license, tenant};

/// License guard middleware.
/// Blocks any request from a tenant that has no active license,
/// UNLESS the tenant is a system tenant (is_system = true).
/// Must be applied AFTER the JWT auth middleware (so Claims are available).
pub async fn check_license(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let db = match state.db.as_ref() {
        Some(db) => db,
        None => return Ok(next.run(req).await),
    };

    let claims = match req.extensions().get::<crate::middleware::auth::Claims>() {
        Some(c) => c.clone(),
        // No claims means JWT middleware already rejected the request
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Always allow system tenants to pass through
    let caller_tenant = tenant::Entity::find_by_id(&claims.tenant_id)
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(t) = caller_tenant {
        if t.is_system {
            return Ok(next.run(req).await);
        }
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // For non-system tenants: require an active, non-revoked license
    let active_license = match license::Entity::find()
        .filter(license::Column::TenantId.eq(&claims.tenant_id))
        .filter(license::Column::IsActive.eq(true))
        .filter(license::Column::RevokedAt.is_null())
        .order_by_desc(license::Column::CreatedAt)
        .one(db)
        .await
    {
        Ok(lic) => lic,
        Err(e) => {
            // Fail-open: if the table doesn't exist yet (e.g. test DB), let through
            tracing::warn!("[LicenseGuard] DB error querying licenses, fail-open: {}", e);
            return Ok(next.run(req).await);
        }
    };

    if active_license.is_none() {
        tracing::warn!(
            "[LicenseGuard] Tenant {} blocked — no active license",
            &claims.tenant_id
        );
        return Err(StatusCode::PAYMENT_REQUIRED);
    }

    let lic = active_license.unwrap();
    let sub = match crate::models::subscription::Entity::find_by_id(&lic.subscription_id)
        .one(db)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("[LicenseGuard] DB error querying subscription, fail-open: {}", e);
            return Ok(next.run(req).await);
        }
    };

    match sub {
        None => {
            tracing::warn!(
                "[LicenseGuard] Tenant {} blocked — subscription not found",
                &claims.tenant_id
            );
            Err(StatusCode::PAYMENT_REQUIRED)
        }
        Some(s) => {
            let now = chrono::Utc::now().fixed_offset();
            if s.status == "suspended" || s.status == "cancelled" || s.expires_at < now {
                tracing::warn!(
                    "[LicenseGuard] Tenant {} blocked — subscription is {} / expired",
                    &claims.tenant_id, s.status
                );
                Err(StatusCode::PAYMENT_REQUIRED)
            } else {
                Ok(next.run(req).await)
            }
        }
    }
}

/// Spawns the 12-hour background task that checks/suspends expired licenses
/// and sends renewal alerts.
pub fn start_license_check_worker(state: Arc<AppState>) {
    tokio::spawn(async move {
        let interval = std::time::Duration::from_secs(12 * 60 * 60);
        loop {
            tracing::info!("[LicenseTask] Running scheduled license validity check…");
            crate::services::license_service::LicenseService::check_and_notify_expiring_licenses(&state).await;
            tokio::time::sleep(interval).await;
        }
    });
}
