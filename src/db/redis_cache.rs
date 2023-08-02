use futures::lock::Mutex;
use redis::{AsyncCommands, RedisResult};
use std::sync::Arc;

/// Initialises a connection to a Redis cache
///
/// ### Arguments
///
/// * `url` - URL of the Redis cache instance to connect to
pub async fn init_cache(url: &str) -> Arc<Mutex<redis::aio::ConnectionManager>> {
    let redis_client = redis::Client::open(url).unwrap();
    let redis_connection_manager = redis::aio::ConnectionManager::new(redis_client)
        .await
        .unwrap();

    Arc::new(Mutex::new(redis_connection_manager))
}

/// Sets a data entry in the cache
///
/// ### Arguments
///
/// * `connection` - Redis cache connection
/// * `key` - Key of the data entry to set
/// * `value` - Value of the data entry to set
pub async fn set_data_in_cache(
    connection: Arc<Mutex<redis::aio::ConnectionManager>>,
    key: &str,
    value: &str,
) -> RedisResult<()> {
    let mut connection_ref = connection.lock().await;
    connection_ref.set(key, value).await
}

/// Gets a data entry from the cache
///
/// ### Arguments
///
/// * `connection` - Redis cache connection
/// * `key` - Key of the data entry to get
pub async fn get_data_from_cache(
    connection: Arc<Mutex<redis::aio::ConnectionManager>>,
    key: &str,
) -> RedisResult<String> {
    let mut connection_ref = connection.lock().await;
    connection_ref.get(key).await
}
