use std::sync::Arc;
use futures::lock::Mutex;
use redis::{AsyncCommands, RedisResult};

/// ======= REDIS FUNCTIONS ======= ///

pub async fn redis_init(url: &str) -> Arc<Mutex<redis::aio::ConnectionManager>> {
    let redis_client = redis::Client::open(url).unwrap();
    let redis_connection_manager = redis::aio::ConnectionManager::new(redis_client).await.unwrap();
    
    Arc::new(Mutex::new(redis_connection_manager))
}

pub async fn redis_set_data(connection: Arc<Mutex<redis::aio::ConnectionManager>>, key: &str, value: &str) -> RedisResult<()> {
    let mut connection_ref = connection.lock().await;
    connection_ref.set(key, value).await
}

pub async fn redis_get_data(connection: Arc<Mutex<redis::aio::ConnectionManager>>, key: &str) -> RedisResult<String> {
    let mut connection_ref = connection.lock().await;
    connection_ref.get(key).await
}