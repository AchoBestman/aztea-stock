use crate::db::create_pool;

#[tokio::test]
async fn test_create_pool_none() {
    let pool = create_pool(&None).await;
    assert!(pool.is_none());
}

#[tokio::test]
async fn test_create_pool_invalid_url() {
    let invalid_url = Some("postgres://invalid_host_name_aztea:5432/non_existent_db".to_string());
    let pool = create_pool(&invalid_url).await;
    assert!(pool.is_none());
}
