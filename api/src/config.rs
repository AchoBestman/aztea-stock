use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: Option<String>,
    pub sqlite_database_url: String,
    pub db_type: String, // "postgres" or "sqlite"
    pub offline: bool,
    pub jwt_secret: String,
    pub port: u16,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let database_url = env::var("DATABASE_URL").ok();
        let sqlite_database_url = env::var("SQLITE_DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://aztea-stock-offline.db?mode=rwc".to_string());
        
        let offline = env::var("OFFLINE")
            .map(|val| val.to_lowercase() == "true" || val == "1")
            .unwrap_or(false);

        let db_type = env::var("DB_TYPE")
            .unwrap_or_else(|_| {
                if let Some(ref url) = database_url {
                    if url.starts_with("sqlite:") {
                        "sqlite".to_string()
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

        Ok(Self {
            database_url,
            sqlite_database_url,
            db_type,
            offline,
            jwt_secret,
            port,
            rust_log,
        })
    }
}
