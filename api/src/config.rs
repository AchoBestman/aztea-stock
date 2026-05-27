use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: Option<String>,
    pub sqlite_database_url: String,
    pub db_type: String,
    pub offline: bool,
    pub jwt_secret: String,
    pub port: u16,
    pub rust_log: String,
    // Queue
    pub redis_url: String,
    /// "tokio_task" (default) | "redis" | "cloudflare"
    pub queue_driver: String,
    // Cloudflare Queue (used when queue_driver = "cloudflare")
    pub cloudflare_queue_id: Option<String>,
    pub cloudflare_account_id: Option<String>,
    pub cloudflare_api_token: Option<String>,
    pub cloudflare_worker_url: Option<String>,
    // System SMTP fallback (from .env)
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_secure: bool,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_from: String,
    // Frontend URL (for reset links)
    pub frontend_url: String,
    // AES encryption key for tenant SMTP credentials
    pub encryption_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let database_url = env::var("DATABASE_URL").ok();
        let sqlite_database_url = env::var("SQLITE_DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://aztea-stock-offline.db?mode=rwc".to_string());

        let offline = env::var("OFFLINE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        let db_type = env::var("DB_TYPE")
            .unwrap_or_else(|_| {
                if let Some(ref url) = database_url {
                    if url.starts_with("sqlite:") {
                        "sqlite".to_string()
                    }
                    else if url.starts_with("mysql:") {
                        "mysql".to_string()
                    } else {
                        "postgres".to_string()
                    }
                } else {
                    "postgres".to_string()
                }
            })
            .to_lowercase();

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default_super_secret_key_for_azteastock_123456".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        // Queue
        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let queue_driver = env::var("QUEUE_DRIVER").unwrap_or_else(|_| "tokio_task".to_string());

        // Cloudflare
        let cloudflare_queue_id = env::var("CLOUDFLARE_QUEUE_ID").ok();
        let cloudflare_account_id = env::var("CLOUDFLARE_ACCOUNT_ID").ok();
        let cloudflare_api_token = env::var("CLOUDFLARE_API_TOKEN").ok();
        let cloudflare_worker_url = env::var("CLOUDFLARE_WORKER_URL").ok();

        // System SMTP fallback
        let smtp_host = env::var("SMTP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let smtp_port = env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(1025);
        let smtp_secure = env::var("SMTP_SECURE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);
        let smtp_user = env::var("SMTP_USER").unwrap_or_else(|_| "null".to_string());
        let smtp_pass = env::var("SMTP_PASS").unwrap_or_else(|_| "null".to_string());
        let smtp_from = env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@aztea.com".to_string());

        let frontend_url =
            env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let encryption_key = env::var("ENCRYPTION_KEY")
            .unwrap_or_else(|_| "a-very-secret-key-32-chars-long-!!".to_string());

        Ok(Self {
            database_url,
            sqlite_database_url,
            db_type,
            offline,
            jwt_secret,
            port,
            rust_log,
            redis_url,
            queue_driver,
            cloudflare_queue_id,
            cloudflare_account_id,
            cloudflare_api_token,
            cloudflare_worker_url,
            smtp_host,
            smtp_port,
            smtp_secure,
            smtp_user,
            smtp_pass,
            smtp_from,
            frontend_url,
            encryption_key,
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: None,
            sqlite_database_url: "sqlite://:memory:".to_string(),
            db_type: "sqlite".to_string(),
            offline: true,
            jwt_secret: "test_jwt_secret_123456_test_jwt_secret".to_string(),
            port: 8080,
            rust_log: "info".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            queue_driver: "tokio_task".to_string(),
            cloudflare_queue_id: None,
            cloudflare_account_id: None,
            cloudflare_api_token: None,
            cloudflare_worker_url: None,
            smtp_host: "127.0.0.1".to_string(),
            smtp_port: 1025,
            smtp_secure: false,
            smtp_user: "null".to_string(),
            smtp_pass: "null".to_string(),
            smtp_from: "noreply@aztea.com".to_string(),
            frontend_url: "http://localhost:3000".to_string(),
            encryption_key: "a-very-secret-key-32-chars-long-!!".to_string(),
        }
    }
}
