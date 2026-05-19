use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{info, warn};

pub async fn create_pool(database_url: &Option<String>) -> Option<PgPool> {
    let url = match database_url {
        Some(url) => url,
        None => {
            warn!("DATABASE_URL env var not found. Database functionality will be disabled.");
            return None;
        }
    };

    match PgPoolOptions::new()
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
            warn!("Failed to connect to PostgreSQL at {}: {}. Database functionality will be disabled.", url, e);
            None
        }
    }
}
