use sqlx::{any::AnyPoolOptions, AnyPool};
use tracing::{info, warn};
use crate::config::Config;

pub async fn create_pool(config: &Config) -> Option<AnyPool> {
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

    sqlx::any::install_default_drivers();
    match AnyPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(url)
        .await
    {
        Ok(pool) => {
            info!("Successfully connected to PostgreSQL database.");
            Some(pool)
        }
        Err(e) => {
            warn!("Failed to connect to PostgreSQL at {}: {}. Falling back to SQLite.", url, e);
            connect_sqlite(&config.sqlite_database_url).await
        }
    }
}

async fn connect_sqlite(url: &str) -> Option<AnyPool> {
    sqlx::any::install_default_drivers();
    match AnyPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(url)
        .await
    {
        Ok(pool) => {
            info!("Successfully connected to SQLite database.");
            Some(pool)
        }
        Err(e) => {
            warn!("Failed to connect to SQLite at {}: {}. Database functionality will be disabled.", url, e);
            None
        }
    }
}
