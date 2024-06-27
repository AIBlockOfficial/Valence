use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;

/// Trait for a key-value data store connection
#[async_trait]
pub trait KvStoreConnection {
    /// Initialize a connection to the cache
    ///
    /// ### Arguments
    ///
    /// * `url` - A string slice that holds the URL to connect to
    async fn init(url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized;

    /// Sets a data entry in the cache
    ///
    /// ### Arguments
    ///
    /// * `key` - Key of the data entry to set
    /// * `value_id` - ID of the value to set
    /// * `value` - Value of the data entry to set
    async fn set_data<T: Serialize + Send + DeserializeOwned>(
        &mut self,
        key: &str,
        value_id: &str,
        value: T,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Sets a data entry in the cache with an expiration time
    ///
    /// ### Arguments
    ///
    /// * `key` - Key of the data entry to set
    /// * `value_id` - ID of the value to set
    /// * `value` - Value of the data entry to set
    /// * `seconds` - Number of seconds to expire the data entry in
    async fn set_data_with_expiry<T: Serialize + DeserializeOwned + Send>(
        &mut self,
        key: &str,
        value_id: &str,
        value: T,
        seconds: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Deletes a data entry from the cache
    ///
    /// ### Arguments
    ///
    /// * `key` - Key of the data entry to delete
    /// * `value_id` - ID of the value to delete. If not provided, all values for the key are deleted
    async fn del_data(
        &mut self,
        key: &str,
        value_id: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Gets a data entry from the cache
    ///
    /// ### Arguments
    ///
    /// * `key` - Key of the data entry to get
    /// * `value_id` - ID of the value to get. If not provided, all values for the key are retrieved
    async fn get_data<T: Clone + DeserializeOwned>(
        &mut self,
        key: &str,
        value_id: Option<&str>,
    ) -> Result<Option<HashMap<String, T>>, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
pub trait CacheHandler {
    /// Sets an expiration time for a data entry
    ///
    /// ### Arguments
    ///
    /// * `key` - Key of the data entry to expire
    /// * `seconds` - Number of seconds to expire the data entry in
    async fn expire_entry(
        &mut self,
        key: &str,
        seconds: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
