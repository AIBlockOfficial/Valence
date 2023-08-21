use serde::{Deserialize, Serialize};
use async_trait::async_trait;

#[async_trait]
pub trait DbConnection {
    type ConnectionResult;
    type InsertCollResult;
    type InsertDocResult;

    /// Initialize a connection to the database
    /// 
    /// ### Arguments
    /// 
    /// * `url` - A string slice that holds the URL to connect to
    async fn init(&mut self, url: &str) -> Self::ConnectionResult;

    /// Insert a collection into the database
    /// 
    /// ### Arguments
    /// 
    /// * `db_name` - A string slice that holds the name of the database
    /// * `coll_name` - A string slice that holds the name of the collection
    async fn insert_collection(&mut self, db_name: &str, coll_name: &str) -> Self::InsertCollResult;

    /// Insert a document into the database
    /// 
    /// ### Arguments
    /// 
    /// * `db_name` - A string slice that holds the name of the database
    /// * `coll_name` - A string slice that holds the name of the collection
    /// * `doc` - A generic type that holds the document to insert
    async fn insert_document<'a, T: Serialize + Deserialize<'a> + std::marker::Send>(
        &mut self,
        db_name: &str,
        coll_name: &str,
        doc: T,
    ) -> Self::InsertDocResult;
}

#[async_trait]
pub trait CacheConnection {
    type ConnectionResult;

    /// Initialize a connection to the cache
    /// 
    /// ### Arguments
    /// 
    /// * `url` - A string slice that holds the URL to connect to
    async fn init(&mut self, url: &str) -> Self::ConnectionResult;
}