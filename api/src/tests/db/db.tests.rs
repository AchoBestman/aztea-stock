use crate::database::create_connection;
use crate::config::Config;
use sea_orm::DatabaseConnection;

#[tokio::test]
async fn test_create_pool_none() {
    let config = Config {
        database_url: None,
        sqlite_database_url: "sqlite://invalid/path/to/db.db".to_string(),
        db_type: "postgres".to_string(),
        offline: false,
        jwt_secret: "secret".to_string(),
        port: 8080,
        rust_log: "info".to_string(),
        ..Config::default()
    };
    let conn: Option<DatabaseConnection> = create_connection(&config).await;
    assert!(conn.is_none());
}

#[tokio::test]
async fn test_create_pool_invalid_url() {
    let config = Config {
        database_url: Some("postgres://invalid_host_name_aztea:5432/non_existent_db".to_string()),
        sqlite_database_url: "sqlite://invalid/path/to/db.db".to_string(),
        db_type: "postgres".to_string(),
        offline: false,
        jwt_secret: "secret".to_string(),
        port: 8080,
        rust_log: "info".to_string(),
        ..Config::default()
    };
    let conn: Option<DatabaseConnection> = create_connection(&config).await;
    assert!(conn.is_none());
}
