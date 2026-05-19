use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;
use tracing::{info, warn};
use crate::config::Config;

pub async fn create_connection(config: &Config) -> Option<DatabaseConnection> {
    // 1. Force SQLite in offline mode
    if config.offline {
        info!("Offline mode active: connecting directly to SQLite database.");
        return connect_sqlite(&config.sqlite_database_url).await;
    }

    // 2. Select database based on DB_TYPE preferred by user in development
    if config.db_type == "sqlite" {
        info!("SQLite selected as preferred database.");
        return connect_sqlite(&config.sqlite_database_url).await;
    }

    // 3. Default to PostgreSQL, fall back to SQLite if PostgreSQL is unavailable
    let url = match &config.database_url {
        Some(url) => url,
        None => {
            warn!("DATABASE_URL env var not found. Falling back to SQLite database.");
            return connect_sqlite(&config.sqlite_database_url).await;
        }
    };

    let mut opt = ConnectOptions::new(url.clone());
    opt.max_connections(5)
       .acquire_timeout(Duration::from_secs(3));

    match Database::connect(opt).await {
        Ok(conn) => {
            info!("Successfully connected to PostgreSQL database.");
            Some(conn)
        }
        Err(e) => {
            warn!("Failed to connect to PostgreSQL at {}: {}. Falling back to SQLite.", url, e);
            connect_sqlite(&config.sqlite_database_url).await
        }
    }
}

async fn connect_sqlite(url: &str) -> Option<DatabaseConnection> {
    let mut opt = ConnectOptions::new(url.to_owned());
    opt.max_connections(5)
       .acquire_timeout(Duration::from_secs(3));

    match Database::connect(opt).await {
        Ok(conn) => {
            info!("Successfully connected to SQLite database.");
            Some(conn)
        }
        Err(e) => {
            warn!("Failed to connect to SQLite at {}: {}. Database functionality will be disabled.", url, e);
            None
        }
    }
}
