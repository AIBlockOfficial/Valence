use crate::{db::handler::KvStoreConnection, utils::deserialize_data, utils::serialize_data};
use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands, RedisError};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone)]
pub struct RedisCacheConn {
    pub connection: ConnectionManager,
}

#[async_trait]
impl KvStoreConnection for RedisCacheConn {
    type ConnectionResult = RedisCacheConn;
    type SetDataResult = Result<(), RedisError>;
    type GetDataResult<T> = Result<Option<T>, RedisError>;

    async fn init(url: &str) -> Self::ConnectionResult {
        let redis_client = redis::Client::open(url).unwrap();
        let redis_connection_manager = ConnectionManager::new(redis_client).await.unwrap();

        RedisCacheConn {
            connection: redis_connection_manager,
        }
    }

    async fn set_data<T: Serialize + Send>(&mut self, key: &str, value: T) -> Self::SetDataResult {
        let serialized_data = serialize_data(&value);
        let _: () = self.connection.set(key, serialized_data).await?;
        Ok(())
    }

    async fn get_data<T: DeserializeOwned>(&mut self, key: &str) -> Self::GetDataResult<T> {
        let result: Option<String> = self.connection.get(key).await?;

        if let Some(data) = result {
            let deserialized: T = deserialize_data(data);
            return Ok(Some(deserialized));
        }

        Ok(None)
    }
}
