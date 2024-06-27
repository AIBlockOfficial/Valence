use std::collections::HashMap;

use crate::db::handler::{CacheHandler, KvStoreConnection};
use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{event, span, Level};

#[derive(Clone)]
pub struct RedisCacheConn {
    pub connection: ConnectionManager,
}

#[async_trait]
impl CacheHandler for RedisCacheConn {
    async fn expire_entry(
        &mut self,
        key: &str,
        seconds: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.connection.expire(key, seconds).await?;
        Ok(())
    }
}

#[async_trait]
impl KvStoreConnection for RedisCacheConn {
    async fn init(url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let redis_client = redis::Client::open(url)?;
        let redis_connection_manager = ConnectionManager::new(redis_client).await?;

        Ok(RedisCacheConn {
            connection: redis_connection_manager,
        })
    }

    async fn set_data<T: Serialize + DeserializeOwned + Send>(
        &mut self,
        key: &str,
        value_id: &str,
        value: T,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let exists: bool = self.connection.exists(key).await?;

        let mut mapping: HashMap<String, T> = if exists {
            // Get the existing data
            let data: String = self.connection.get(key).await?;
            serde_json::from_str(&data)?
        } else {
            HashMap::new()
        };

        // Append the new data to the vec
        mapping.insert(value_id.to_string(), value);

        let serialized = serde_json::to_string(&mapping)?;
        self.connection.set(key, serialized).await?;

        Ok(())
    }

    async fn set_data_with_expiry<T: Serialize + DeserializeOwned + Send>(
        &mut self,
        key: &str,
        value_id: &str,
        value: T,
        seconds: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if the key exists
        let exists: bool = self.connection.exists(key).await?;

        let mut mapping: HashMap<String, T> = if exists {
            // Get the existing data
            let data: String = self.connection.get(key).await?;
            serde_json::from_str(&data)?
        } else {
            HashMap::new()
        };

        // Append the new data to the hashmap
        mapping.insert(value_id.to_string(), value);

        // Serialize the vec back to a string
        let serialized = serde_json::to_string(&mapping)?;

        // Set the data back to Redis
        self.connection.set(key, serialized).await?;

        // Set the expiry time for the key
        self.connection.expire(key, seconds).await?;

        Ok(())
    }

    async fn del_data(
        &mut self,
        key: &str,
        value_id: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(value_id) = value_id {
            let exists: bool = self.connection.exists(key).await?;

            if exists {
                let mut mapping: HashMap<String, String> = self.get_data(key, None).await?.unwrap();
                mapping.remove(value_id);
                let serialized = serde_json::to_string(&mapping)?;
                self.connection.set(key, serialized).await?;
            }
            return Ok(());
        }

        let _: () = self.connection.del(key).await?;
        Ok(())
    }

    async fn get_data<T: Clone + DeserializeOwned>(
        &mut self,
        key: &str,
        value_id: Option<&str>,
    ) -> Result<Option<HashMap<String, T>>, Box<dyn std::error::Error + Send + Sync>> {
        let span = span!(Level::TRACE, "MongoDbConn::get_data");
        let _enter = span.enter();

        // Check if the key exists
        let exists: bool = self.connection.exists(key).await?;

        if exists {
            // Get the existing data
            let data: String = self.connection.get(key).await?;
            let mapping: HashMap<String, T> = serde_json::from_str(&data)?;

            if let Some(value_id) = value_id {
                let value = mapping.get(value_id);
                if let Some(value) = value {
                    let mut new_mapping: HashMap<String, T> = HashMap::new();
                    new_mapping.insert(value_id.to_string(), value.clone());
                    return Ok(Some(new_mapping));
                } else {
                    // Value with the given ID not found
                    event!(
                        Level::ERROR,
                        "Value with ID {value_id} not found for key {key}"
                    );
                    return Ok(None);
                }
            }

            return Ok(Some(mapping));
        }

        Ok(None)
    }
}
