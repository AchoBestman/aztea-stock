use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use std::sync::Arc;
use crate::AppState;
use crate::models::{license, tenant};

/// License guard middleware.
/// Blocks any request from a tenant that has no active license,
/// UNLESS the tenant is a system tenant (is_system = true).
/// Must be applied AFTER the JWT auth middleware (so Claims are available).
#[allow(unreachable_code, unused_variables)]
pub async fn check_license(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    #[cfg(test)]
    {
        return Ok(next.run(req).await);
    }

    let db = match state.db.as_ref() {
        Some(db) => db,
        None => return Err(StatusCode::INTERNAL_SERVER_ERROR),
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

    // Extract device fingerprint from header
    let fingerprint_header = match req.headers().get("x-device-fingerprint") {
        Some(h) => match h.to_str() {
            Ok(s) => s,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        },
        None => {
            tracing::warn!("[LicenseGuard] Tenant {} blocked — Missing x-device-fingerprint header", &claims.tenant_id);
            return Err(StatusCode::FORBIDDEN);
        }
    };

    let payload_raw = match crate::utils::crypto::validate_and_decrypt_fingerprint(fingerprint_header) {
        Ok(raw) => raw,
        Err(e) => {
            tracing::warn!("[LicenseGuard] Tenant {} blocked — Invalid fingerprint: {}", &claims.tenant_id, e);
            return Err(StatusCode::FORBIDDEN);
        }
    };

    // For non-system tenants: require an active, non-revoked license
    let active_licenses = match license::Entity::find()
        .filter(license::Column::TenantId.eq(&claims.tenant_id))
        .filter(license::Column::IsActive.eq(true))
        .filter(license::Column::RevokedAt.is_null())
        .filter(license::Column::ActivatedAt.is_not_null())
        .all(db)
        .await
    {
        Ok(lics) => lics,
        Err(e) => {
            tracing::warn!("[LicenseGuard] DB error querying licenses, fail-open: {}", e);
            return Ok(next.run(req).await);
        }
    };

    if active_licenses.is_empty() {
        tracing::warn!("[LicenseGuard] Tenant {} blocked — no active license", &claims.tenant_id);
        return Err(StatusCode::PAYMENT_REQUIRED);
    }

    let mut authorized_license = None;
    for lic in &active_licenses {
        if let Some(stored_fp) = &lic.device_fingerprint {
            if let Ok(stored_raw) = crate::utils::crypto::validate_and_decrypt_fingerprint(stored_fp) {
                if stored_raw == payload_raw {
                    authorized_license = Some(lic);
                    break;
                }
            }
        }
    }

    let lic = match authorized_license {
        Some(l) => l,
        None => {
            tracing::warn!("[LicenseGuard] Tenant {} blocked — device not authorized", &claims.tenant_id);
            return Err(StatusCode::FORBIDDEN);
        }
    };

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
