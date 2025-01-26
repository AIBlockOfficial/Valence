use std::collections::HashMap;

use crate::db::handler::{CacheHandler, KvStoreConnection};
use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{event, span, Level};

/// A Redis-backed cache connection handling key-value storage and connection lifecycle.
///
/// Manages connections to Redis, providing methods to set, retrieve, and delete
/// serialized data. Automatically handles connection management and expiration.
#[derive(Clone)]
pub struct RedisCacheConn {
    pub connection: ConnectionManager,
}

#[async_trait]
impl CacheHandler for RedisCacheConn {
    /// Implements cache expiration functionality for Redis keys.
    ///
    /// Provides atomic expiration commands through the Redis EXPIRE interface.
    async fn expire_entry(
        &mut self,
        key: &str,
        seconds: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.connection.expire(key, seconds).await?;
        Ok(())
    }
}

/// Core connection handler implementing key-value store operations.
///
/// Manages Redis connections and implements CRUD operations with JSON serialization.
/// Handles connection lifecycle through `init` method.
#[async_trait]
impl KvStoreConnection for RedisCacheConn {
    async fn init(url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let redis_client = redis::Client::open(url)?;
        let redis_connection_manager = ConnectionManager::new(redis_client).await?;

        Ok(RedisCacheConn {
            connection: redis_connection_manager,
        })
    }

    /// Stores a value in the cache without setting expiration.
    ///
    /// # Arguments
    /// * `key` - Redis key for the hashmap
    /// * `value_id` - Field within the hashmap
    /// * `value` - Serializable value to store
    ///
    /// # Serialization
    /// Values are serialized to JSON using `serde_json`. Ensure `T` implements
    /// `Serialize` and `DeserializeOwned`.
    ///
    /// # Errors
    /// Returns errors on connection failures, serialization issues, or Redis operation failures.
    async fn set_data<T: Serialize + DeserializeOwned + Send>(
        &mut self,
        key: &str,
        value_id: &str,
        value: T,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let exists: bool = self.connection.exists(key).await?;

        let mut mapping: HashMap<String, T> = if exists {
            let data: String = self.connection.get(key).await?;
            serde_json::from_str(&data)?
        } else {
            HashMap::new()
        };

        mapping.insert(value_id.to_string(), value);

        let serialized = serde_json::to_string(&mapping)?;
        self.connection.set(key, serialized).await?;

        Ok(())
    }

    /// Stores a value with expiration time (TTL).
    ///
    /// # Arguments
    /// * `key` - Redis key for the hashmap
    /// * `value_id` - Field within the hashmap
    /// * `value` - Serializable value to store
    /// * `seconds` - TTL in seconds for automatic expiration
    ///
    /// The TTL applies to the entire Redis key. Subsequent updates to the hashmap
    /// will maintain the TTL unless explicitly modified.
    async fn set_data_with_expiry<T: Serialize + DeserializeOwned + Send>(
        &mut self,
        key: &str,
        value_id: &str,
        value: T,
        seconds: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let exists: bool = self.connection.exists(key).await?;

        let mut mapping: HashMap<String, T> = if exists {
            let data: String = self.connection.get(key).await?;
            serde_json::from_str(&data)?
        } else {
            HashMap::new()
        };

        mapping.insert(value_id.to_string(), value);

        let serialized = serde_json::to_string(&mapping)?;
        self.connection.set(key, serialized).await?;
        self.connection.expire(key, seconds).await?;

        Ok(())
    }

    /// Deletes cache entries with configurable scope.
    ///
    /// When `value_id` is provided:
    /// - Performs non-atomic read-modify-write operation on the hashmap
    /// - May impact performance with large datasets due to full value retrieval
    ///
    /// When `value_id` is `None`:
    /// - Deletes entire key atomically via Redis DEL command
    /// - Optimal for bulk deletion operations
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

    /// Retrieves data from the cache.
    ///
    /// # Arguments
    /// * `key` - Redis key to retrieve
    /// * `value_id` - Optional specific field to extract from hashmap
    ///
    /// Returns `None` if key doesn't exist or specific value_id not found.
    /// Deserialization errors and connection issues will return error variants.
    async fn get_data<T: Clone + DeserializeOwned>(
        &mut self,
        key: &str,
        value_id: Option<&str>,
    ) -> Result<Option<HashMap<String, T>>, Box<dyn std::error::Error + Send + Sync>> {
        let span = span!(Level::TRACE, "MongoDbConn::get_data");
        let _enter = span.enter();

        let exists: bool = self.connection.exists(key).await?;

        if exists {
            let data: String = self.connection.get(key).await?;
            let mapping: HashMap<String, T> = serde_json::from_str(&data)?;

            if let Some(value_id) = value_id {
                let value = mapping.get(value_id);
                if let Some(value) = value {
                    let mut new_mapping: HashMap<String, T> = HashMap::new();
                    new_mapping.insert(value_id.to_string(), value.clone());
                    return Ok(Some(new_mapping));
                } else {
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