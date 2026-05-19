use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: Option<String>,
    pub jwt_secret: String,
    pub port: u16,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let database_url = env::var("DATABASE_URL").ok();
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default_super_secret_key_for_azteastock_123456".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Ok(Self {
            database_url,
            jwt_secret,
            port,
            rust_log,
        })
    }
}
