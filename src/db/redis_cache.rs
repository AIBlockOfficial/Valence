use futures::lock::Mutex;
use redis::{AsyncCommands, RedisResult};
use std::sync::Arc;

pub async fn init_cache(url: &str) -> Arc<Mutex<redis::aio::ConnectionManager>> {
    let redis_client = redis::Client::open(url).unwrap();
    let redis_connection_manager = redis::aio::ConnectionManager::new(redis_client)
        .await
        .unwrap();

    Arc::new(Mutex::new(redis_connection_manager))
}

pub async fn set_data_in_cache(
    connection: Arc<Mutex<redis::aio::ConnectionManager>>,
    key: &str,
    value: &str,
) -> RedisResult<()> {
    let mut connection_ref = connection.lock().await;
    connection_ref.set(key, value).await
}

pub async fn get_data_from_cache(
    connection: Arc<Mutex<redis::aio::ConnectionManager>>,
    key: &str,
) -> RedisResult<String> {
    let mut connection_ref = connection.lock().await;
    connection_ref.get(key).await
}
