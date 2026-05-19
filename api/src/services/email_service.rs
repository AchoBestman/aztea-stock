use std::sync::Arc;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use sea_orm::{EntityTrait};

use crate::AppState;
use crate::utils::crypto::decrypt;

/// ─── Email Job ──────────────────────────────────────────────────────────────
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailJob {
    pub id: String,
    pub tenant_id: String,
    pub to: String,
    pub subject: String,
    pub html: String,
    pub attempts: u32,
    pub max_attempts: u32,
    pub scheduled_at: String,
    pub created_at: String,
}

/// SMTP credentials resolved from a Tenant or the system .env fallback
struct SmtpConfig {
    host: String,
    port: u16,
    secure: bool,
    user: Option<String>,
    pass: Option<String>,
    from: String,
}

// ─── SMTP resolution (mirrors aztea-store getCompanyTransporter) ─────────────
/// Resolves SMTP settings for a given tenant_id.
/// Priority: tenant SMTP fields → system tenant SMTP fields → .env fallback.
async fn resolve_smtp(state: &AppState, tenant_id: &str) -> SmtpConfig {
    let fallback = SmtpConfig {
        host: state.config.smtp_host.clone(),
        port: state.config.smtp_port,
        secure: state.config.smtp_secure,
        user: if state.config.smtp_user == "null" { None } else { Some(state.config.smtp_user.clone()) },
        pass: if state.config.smtp_pass == "null" { None } else { Some(state.config.smtp_pass.clone()) },
        from: state.config.smtp_from.clone(),
    };

    let db = match state.db.as_ref() {
        Some(db) => db,
        None => return fallback,
    };

    // Try the requested tenant first
    let tenant = crate::models::tenant::Entity::find_by_id(tenant_id)
        .one(db)
        .await
        .ok()
        .flatten();

    let source = match tenant {
        Some(ref t) if t.sender_email.is_some() && t.sender_user.is_some() && t.sender_password.is_some() => {
            Some(t.clone())
        }
        _ => {
            // Fall back to the system tenant
            use sea_orm::{QueryFilter, ColumnTrait};
            crate::models::tenant::Entity::find()
                .filter(crate::models::tenant::Column::IsSystem.eq(true))
                .one(db)
                .await
                .ok()
                .flatten()
        }
    };

    match source {
        Some(t) if t.sender_email.is_some() && t.sender_user.is_some() && t.sender_password.is_some() => {
            let from = t.sender_email.clone().unwrap();
            let user = decrypt(t.sender_user.as_deref().unwrap_or(""));
            let pass = decrypt(t.sender_password.as_deref().unwrap_or(""));
            SmtpConfig {
                host: state.config.smtp_host.clone(),
                port: state.config.smtp_port,
                secure: state.config.smtp_secure,
                user: Some(user),
                pass: Some(pass),
                from,
            }
        }
        _ => fallback,
    }
}

// ─── Direct SMTP send ────────────────────────────────────────────────────────
/// Sends the email immediately via SMTP (lettre), using tenant SMTP settings
/// with a transparent .env fallback — mirrors aztea-store sendEmailDirect().
pub async fn send_email_direct(
    state: &AppState,
    tenant_id: &str,
    to: &str,
    subject: &str,
    html: &str,
) -> Result<(), anyhow::Error> {
    let smtp = resolve_smtp(state, tenant_id).await;

    let email = Message::builder()
        .from(smtp.from.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(html.to_string())?;

    // Secure = true → STARTTLS/TLS relay (production), false → plain (Mailpit/dev)
    let transport = if smtp.secure {
        let mut builder = SmtpTransport::relay(&smtp.host)
            .map_err(|e| anyhow::anyhow!("SMTP relay error: {}", e))?
            .port(smtp.port);
        if let (Some(user), Some(pass)) = (smtp.user.clone(), smtp.pass.clone()) {
            builder = builder.credentials(Credentials::new(user, pass));
        }
        builder.build()
    } else {
        let mut builder = SmtpTransport::builder_dangerous(&smtp.host).port(smtp.port);
        if let (Some(user), Some(pass)) = (smtp.user.clone(), smtp.pass.clone()) {
            builder = builder.credentials(Credentials::new(user, pass));
        }
        builder.build()
    };

    let email_clone = email.clone();
    tokio::task::spawn_blocking(move || transport.send(&email_clone))
        .await??;

    Ok(())
}

// ─── Enqueue ─────────────────────────────────────────────────────────────────
/// Enqueues an email according to QUEUE_DRIVER:
///   "tokio_task"  — spawns a background tokio task (default)
///   "redis"       — pushes to Redis `email_queue` list
///   "cloudflare"  — POSTs to the Cloudflare Queue API
pub async fn enqueue_email(
    state: &AppState,
    tenant_id: &str,
    to: &str,
    subject: &str,
    html: &str,
) -> Result<bool, anyhow::Error> {
    let job = EmailJob {
        id: uuid::Uuid::new_v4().to_string(),
        tenant_id: tenant_id.to_string(),
        to: to.to_string(),
        subject: subject.to_string(),
        html: html.to_string(),
        attempts: 0,
        max_attempts: 3,
        scheduled_at: chrono::Utc::now().to_rfc3339(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    match state.config.queue_driver.as_str() {
        // ── Redis ──────────────────────────────────────────────────────────
        "redis" => {
            let client = redis::Client::open(state.config.redis_url.clone())?;
            let mut conn = client.get_multiplexed_tokio_connection().await?;
            use redis::AsyncCommands;
            conn.lpush::<_, _, ()>("email_queue", serde_json::to_string(&job)?).await?;
            tracing::info!("📥 [Redis] Email enqueued → {}", to);
            Ok(true)
        }

        // ── Cloudflare Queue ───────────────────────────────────────────────
        "cloudflare" => {
            let account_id = state.config.cloudflare_account_id.as_deref().unwrap_or("");
            let queue_id   = state.config.cloudflare_queue_id.as_deref().unwrap_or("");
            let api_token  = state.config.cloudflare_api_token.as_deref().unwrap_or("");

            if account_id.is_empty() || queue_id.is_empty() || api_token.is_empty() {
                tracing::error!("❌ Cloudflare queue config incomplete — check CLOUDFLARE_* env vars");
                return Ok(false);
            }

            let payload = serde_json::json!({ "body": job });
            let serialized = serde_json::to_string(&payload)?;
            let payload_size = serialized.len();

            // Cloudflare limit: 128 KB
            if payload_size > 128 * 1024 {
                tracing::error!("❌ Cloudflare Queue message too large ({} bytes). Limit is 128 KB.", payload_size);
                // Graceful fallback to tokio_task
                return dispatch_tokio_task(state, job).await;
            }

            let url = format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/queues/{}/messages",
                account_id, queue_id
            );

            let client = reqwest::Client::new();
            let res = client.post(&url)
                .bearer_auth(api_token)
                .header("Content-Type", "application/json")
                .body(serialized)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Cloudflare Queue request failed: {}", e))?;

            if res.status().is_success() {
                tracing::info!("📤 [Cloudflare Queue] Email enqueued → {}", to);
                Ok(true)
            } else {
                let status = res.status();
                let body = res.text().await.unwrap_or_default();
                tracing::error!("❌ Cloudflare Queue API Error ({}): {}", status, body);
                Ok(false)
            }
        }

        // ── Tokio Task (default) ───────────────────────────────────────────
        _ => dispatch_tokio_task(state, job).await,
    }
}

/// Spawns a detached tokio task to send the email immediately
async fn dispatch_tokio_task(state: &AppState, job: EmailJob) -> Result<bool, anyhow::Error> {
    // Clone what the background task needs
    let state_arc = Arc::new(AppState {
        db: state.db.clone(),
        config: state.config.clone(),
    });
    tokio::spawn(async move {
        match send_email_direct(&state_arc, &job.tenant_id, &job.to, &job.subject, &job.html).await {
            Ok(_) => tracing::info!("✅ [TokioTask] Email sent → {}", job.to),
            Err(e) => tracing::error!("❌ [TokioTask] Email failed → {}: {}", job.to, e),
        }
    });
    Ok(true)
}

// ─── Redis background worker ─────────────────────────────────────────────────
/// Spawns a long-running loop that consumes `email_queue` from Redis
/// and sends each job via SMTP. Only active when QUEUE_DRIVER=redis.
pub fn start_email_worker(state: Arc<AppState>) {
    match state.config.queue_driver.as_str() {
        "redis" => {
            tracing::info!("📬 Starting Redis email queue worker…");
        }
        "cloudflare" => {
            tracing::info!("📬 Queue driver = cloudflare. Worker handled by CF Worker — no local loop needed.");
            return;
        }
        _ => {
            tracing::info!("📬 Queue driver = tokio_task. No background worker needed.");
            return;
        }
    }

    tokio::spawn(async move {
        let client = match redis::Client::open(state.config.redis_url.clone()) {
            Ok(c) => c,
            Err(e) => { tracing::error!("Redis client error: {}", e); return; }
        };
        let mut conn = match client.get_multiplexed_tokio_connection().await {
            Ok(c) => c,
            Err(e) => { tracing::error!("Redis connection error: {}", e); return; }
        };

        use redis::AsyncCommands;
        loop {
            let raw: Option<String> = match conn.rpop("email_queue", None).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Redis rpop error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            if let Some(raw) = raw {
                let mut job: EmailJob = match serde_json::from_str(&raw) {
                    Ok(j) => j,
                    Err(e) => { tracing::error!("Failed to parse email job: {}", e); continue; }
                };

                tracing::info!("Processing email job {} → {}", job.id, job.to);

                match send_email_direct(&state, &job.tenant_id, &job.to, &job.subject, &job.html).await {
                    Ok(_) => tracing::info!("✅ Email job {} sent.", job.id),
                    Err(e) => {
                        job.attempts += 1;
                        tracing::error!("❌ Email job {} failed (attempt {}/{}): {}", job.id, job.attempts, job.max_attempts, e);
                        if job.attempts < job.max_attempts {
                            let _ = conn.lpush::<_, _, ()>("email_queue", serde_json::to_string(&job).unwrap()).await;
                            tracing::warn!("↩ Re-enqueued job {}", job.id);
                        } else {
                            tracing::error!("💀 Email job {} permanently failed.", job.id);
                        }
                    }
                }
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
    });
}

// ─── Email templates ──────────────────────────────────────────────────────────

fn base_template(tenant_name: &str, title: &str, preheader: &str, body: &str) -> String {
    let year = chrono::Utc::now().format("%Y");
    format!(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="UTF-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
  <title>{title}</title>
</head>
<body style="margin:0;padding:0;background:#f4f4f5;font-family:'Helvetica Neue',Arial,sans-serif;">
  <span style="display:none;max-height:0;overflow:hidden;">{preheader}</span>
  <table width="100%" cellpadding="0" cellspacing="0" style="background:#f4f4f5;padding:40px 0;">
    <tr><td align="center">
      <table width="560" cellpadding="0" cellspacing="0" style="background:#ffffff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.08);">
        <!-- Header -->
        <tr>
          <td style="background:#0f172a;padding:24px 40px;">
            <table width="100%" cellpadding="0" cellspacing="0"><tr>
              <td><span style="font-size:18px;font-weight:700;color:#ffffff;letter-spacing:-0.5px;">{tenant_name}</span></td>
              <td align="right"><span style="font-size:11px;color:#94a3b8;text-transform:uppercase;letter-spacing:0.5px;">Sécurisé par Aztea</span></td>
            </tr></table>
          </td>
        </tr>
        <!-- Body -->
        <tr><td style="padding:40px;">{body}</td></tr>
        <!-- Footer -->
        <tr>
          <td style="background:#f8fafc;padding:20px 40px;border-top:1px solid #e2e8f0;">
            <p style="margin:0;font-size:12px;color:#94a3b8;text-align:center;">© {year} {tenant_name} · Tous droits réservés</p>
          </td>
        </tr>
      </table>
    </td></tr>
  </table>
</body>
</html>"#
    )
}

/// Resolves the display name of a tenant for use in email templates.
async fn tenant_name(state: &AppState, tenant_id: &str) -> String {
    let db = match state.db.as_ref() {
        Some(db) => db,
        None => return "Aztea Stock".to_string(),
    };
    crate::models::tenant::Entity::find_by_id(tenant_id)
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|t| t.name)
        .unwrap_or_else(|| "Aztea Stock".to_string())
}

/// OTP 2FA verification email — mirrors sendOTPEmail() from aztea-store
pub async fn send_otp_email(
    state: &AppState,
    tenant_id: &str,
    to: &str,
    code: &str,
) -> Result<bool, anyhow::Error> {
    let name = tenant_name(state, tenant_id).await;
    let body = format!(
        r#"<h2 style="margin:0 0 8px;font-size:22px;color:#0f172a;">Code de vérification</h2>
<p style="margin:0 0 32px;font-size:14px;color:#64748b;">Utilisez ce code pour vous connecter à votre compte {name}.</p>
<div style="background:#f1f5f9;border-radius:10px;padding:32px;text-align:center;margin-bottom:28px;">
  <p style="margin:0 0 12px;font-size:13px;color:#64748b;text-transform:uppercase;letter-spacing:1px;">Votre code</p>
  <div style="font-size:40px;font-weight:800;letter-spacing:12px;color:#140066;font-family:monospace;">{code}</div>
  <p style="margin:16px 0 0;font-size:12px;color:#94a3b8;">⏱ Expire dans <strong>5 minutes</strong></p>
</div>"#
    );
    let html = base_template(&name, "Code de vérification", &format!("Votre code de connexion : {}", code), &body);
    enqueue_email(state, tenant_id, to, &format!("{} — Code de vérification", name), &html).await
}

/// Account creation / password reset invitation email — mirrors sendPasswordResetEmail()
pub async fn send_password_reset_email(
    state: &AppState,
    tenant_id: &str,
    to: &str,
    code: &str,
) -> Result<bool, anyhow::Error> {
    let name = tenant_name(state, tenant_id).await;
    let reset_link = format!(
        "{}/reset-password?email={}&code={}",
        state.config.frontend_url,
        urlencoding::encode(to),
        code
    );
    let body = format!(
        r#"<h2 style="margin:0 0 8px;font-size:22px;color:#0f172a;">Initialisation de votre compte</h2>
<p style="margin:0 0 16px;font-size:14px;color:#64748b;">Vous avez été invité ou avez demandé à réinitialiser votre mot de passe pour {name}.</p>
<div style="background:#f1f5f9;border-radius:10px;padding:32px;text-align:center;margin-bottom:28px;">
  <p style="margin:0 0 12px;font-size:13px;color:#64748b;text-transform:uppercase;letter-spacing:1px;">Votre code OTP</p>
  <div style="font-size:40px;font-weight:800;letter-spacing:12px;color:#140066;font-family:monospace;">{code}</div>
  <p style="margin:16px 0 0;font-size:12px;color:#94a3b8;">⏱ Expire dans <strong>1 heure</strong></p>
</div>
<p style="margin:0 0 24px;font-size:14px;color:#64748b;text-align:center;">Ou cliquez sur le bouton ci-dessous pour continuer depuis un navigateur :</p>
<div style="text-align:center;margin-bottom:32px;">
  <a href="{reset_link}" style="background:#140066;color:#ffffff;padding:14px 28px;border-radius:8px;text-decoration:none;font-weight:600;display:inline-block;">
    Définir mon mot de passe
  </a>
</div>"#
    );
    let html = base_template(&name, "Initialisation du mot de passe", "Définir votre mot de passe", &body);
    enqueue_email(state, tenant_id, to, &format!("{} — Initialisation de votre compte", name), &html).await
}

/// License expiry renewal alert email
pub async fn send_license_renewal_alert(
    state: &AppState,
    tenant_id: &str,
    to: &str,
    plan: &str,
    days_left: i64,
    expires_at: &str,
) -> Result<bool, anyhow::Error> {
    let name = tenant_name(state, tenant_id).await;
    let urgency_color = if days_left <= 3 { "#dc2626" } else { "#d97706" };
    let urgency_label = if days_left <= 3 {
        format!("⚠️ URGENT — {days_left} jour(s) restant(s) !")
    } else {
        format!("🔔 Renouvellement dans {days_left} jour(s)")
    };
    let body = format!(
        r#"<h2 style="margin:0 0 8px;font-size:22px;color:#0f172a;">Votre licence expire bientôt</h2>
<p style="margin:0 0 16px;font-size:14px;color:#64748b;">Cher client de {name}, votre abonnement <strong>{plan}</strong> arrive à expiration.</p>
<div style="background:#fef9c3;border:2px solid {urgency_color};border-radius:10px;padding:24px;text-align:center;margin-bottom:28px;">
  <p style="margin:0;font-size:20px;font-weight:700;color:{urgency_color};">{urgency_label}</p>
  <p style="margin:12px 0 0;font-size:13px;color:#64748b;">Date d'expiration : <strong>{expires_at}</strong></p>
</div>
<p style="margin:0 0 16px;font-size:14px;color:#64748b;">
  Pour éviter toute interruption de service, veuillez contacter votre gestionnaire de compte ou renouveler votre abonnement.
</p>"#
    );
    let html = base_template(&name, "Renouvellement de licence", "Renouvelez votre abonnement", &body);
    enqueue_email(state, tenant_id, to, &format!("{} — Renouvellement de votre licence", name), &html).await
}
