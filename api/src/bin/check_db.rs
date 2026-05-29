use sqlx::mysql::MySqlPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").unwrap();
    let pool = MySqlPoolOptions::new().connect(&db_url).await?;
    let rows = sqlx::query!("SHOW COLUMNS FROM licenses").fetch_all(&pool).await?;
    for row in rows {
        println!("Column: {:?}", row.Field);
    }
    Ok(())
}
