use crate::config::Config;
use std::sync::Mutex;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_config_from_env_defaults() {
    let _guard = ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::remove_var("PORT");
        std::env::remove_var("JWT_SECRET");
        std::env::remove_var("OFFLINE");
        std::env::remove_var("DB_TYPE");
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("SQLITE_DATABASE_URL");
        std::env::remove_var("RUST_LOG");
    }
    let config = Config::from_env();
    assert!(config.is_ok());
    let c = config.unwrap();
    assert!(!c.jwt_secret.is_empty());
    assert_eq!(c.offline, false);
    assert_eq!(c.sqlite_database_url, "sqlite://aztea-stock-offline.db?mode=rwc");
    assert_eq!(c.db_type, "postgres");
}

#[test]
fn test_config_from_env_custom() {
    let _guard = ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::set_var("PORT", "9999");
        std::env::set_var("JWT_SECRET", "custom_secret_key");
        std::env::set_var("DATABASE_URL", "sqlite://custom_path.db");
        std::env::set_var("SQLITE_DATABASE_URL", "sqlite://custom_sqlite_fallback.db");
        std::env::set_var("DB_TYPE", "sqlite");
        std::env::set_var("OFFLINE", "true");
        std::env::set_var("RUST_LOG", "debug");
    }

    let config = Config::from_env().unwrap();
    assert_eq!(config.port, 9999);
    assert_eq!(config.jwt_secret, "custom_secret_key");
    assert_eq!(config.database_url, Some("sqlite://custom_path.db".to_string()));
    assert_eq!(config.sqlite_database_url, "sqlite://custom_sqlite_fallback.db");
    assert_eq!(config.db_type, "sqlite");
    assert_eq!(config.offline, true);
    assert_eq!(config.rust_log, "debug");

    // Clean up env vars
    unsafe {
        std::env::remove_var("PORT");
        std::env::remove_var("JWT_SECRET");
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("SQLITE_DATABASE_URL");
        std::env::remove_var("DB_TYPE");
        std::env::remove_var("OFFLINE");
        std::env::remove_var("RUST_LOG");
    }
}
