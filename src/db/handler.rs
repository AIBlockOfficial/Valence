use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

/// Trait for a key-value data store connection
#[async_trait]
pub trait KvStoreConnection {
    type ConnectionResult;
    type SetDataResult;
    type GetDataResult<T>;

    /// Initialize a connection to the cache
    ///
    /// ### Arguments
    ///
    /// * `url` - A string slice that holds the URL to connect to
    async fn init(url: &str) -> Self::ConnectionResult;

    /// Sets a data entry in the cache
    ///
    /// ### Arguments
    ///
    /// * `key` - Key of the data entry to set
    /// * `value` - Value of the data entry to set
    async fn set_data<T: Serialize + Send>(&mut self, key: &str, value: T) -> Self::SetDataResult;

    /// Gets a data entry from the cache
    ///
    /// ### Arguments
    ///
    /// * `key` - Key of the data entry to get
    async fn get_data<T: DeserializeOwned>(&mut self, key: &str) -> Self::GetDataResult<T>;
}
