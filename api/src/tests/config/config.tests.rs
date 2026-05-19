use crate::config::Config;

#[test]
fn test_config_from_env_defaults() {
    let config = Config::from_env();
    assert!(config.is_ok());
    let c = config.unwrap();
    assert!(!c.jwt_secret.is_empty());
}

#[test]
fn test_config_from_env_custom() {
    unsafe {
        std::env::set_var("PORT", "9999");
        std::env::set_var("JWT_SECRET", "custom_secret_key");
        std::env::set_var("DATABASE_URL", "postgres://localhost/test");
        std::env::set_var("RUST_LOG", "debug");
    }

    let config = Config::from_env().unwrap();
    assert_eq!(config.port, 9999);
    assert_eq!(config.jwt_secret, "custom_secret_key");
    assert_eq!(config.database_url, Some("postgres://localhost/test".to_string()));
    assert_eq!(config.rust_log, "debug");

    // Clean up env vars
    unsafe {
        std::env::remove_var("PORT");
        std::env::remove_var("JWT_SECRET");
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("RUST_LOG");
    }
}
