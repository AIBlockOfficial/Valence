use async_trait::async_trait;
use mongodb::bson::{doc, DateTime, Document};
use mongodb::{options::ClientOptions, Client};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use tracing::{debug, event, span, trace, Level};

use super::handler::KvStoreConnection;

#[derive(Debug, Clone)]
pub struct MongoDbIndex {
    pub db_name: String,
    pub coll_name: String,
}

#[derive(Debug, Clone)]
pub struct MongoDbConn {
    pub client: Client,
    pub index: MongoDbIndex,
}

impl MongoDbConn {
    /// Creates a TTL index on the expiry field.
    ///
    /// NOTE: This function will need to be called in the main function when initialising a MongoDB connection.
    pub async fn create_ttl_index(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let collection = self
            .client
            .database(&self.index.db_name)
            .collection::<Document>(&self.index.coll_name);

        // Create TTL index on the 'expiry' field
        let index_model = mongodb::IndexModel::builder()
            .keys(doc! { "expiry": 1 })
            .options(Some(
                mongodb::options::IndexOptions::builder()
                    .expire_after(Some(std::time::Duration::from_secs(0)))
                    .build(),
            ))
            .build();

        collection.create_index(index_model, None).await?;
        Ok(())
    }
}

#[async_trait]
impl KvStoreConnection for MongoDbConn {
    async fn init(url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Tracing
        let span = span!(Level::TRACE, "MongoDbConn::init");
        let _enter = span.enter();

        let client_options = match ClientOptions::parse(url).await {
            Ok(client_options) => client_options,
            Err(e) => panic!("Failed to connect to MongoDB instance with error: {e}"),
        };

        trace!("Connected to MongoDB instance at {url}");

        let client = match Client::with_options(client_options) {
            Ok(client) => client,
            Err(e) => panic!("Failed to connect to MongoDB instance with error: {e}"),
        };

        trace!("MongoDB client created successfully");

        let index = MongoDbIndex {
            db_name: String::from("default"),
            coll_name: String::from("default"),
        };

        Ok(MongoDbConn { client, index })
    }

    async fn set_data<T: Serialize + std::marker::Send + DeserializeOwned>(
        &mut self,
        key: &str,
        value_id: &str,
        value: T,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("set_data {:?} / {}", key, value_id);
        // Tracing
        let span = span!(Level::TRACE, "MongoDbConn::set_data");
        let _enter = span.enter();

        let collection = self
            .client
            .database(&self.index.db_name)
            .collection::<Document>(&self.index.coll_name);

        // Check if the document with the given key exists
        let filter = doc! { "_id": key };
        let existing_doc: Option<Document> = collection.find_one(filter.clone(), None).await?;

        let mut mapping: HashMap<String, T> = if let Some(doc) = existing_doc {
            if doc.contains_key("data") {
                // Deserialize the existing data
                mongodb::bson::from_bson(doc.get("data").unwrap().clone())?
            } else {
                debug!("No existing data");
                HashMap::new()
            }
        } else {
            debug!("No existing data");
            HashMap::new()
        };

        // Append the new data to the vec
        mapping.insert(value_id.to_string(), value);

        // Serialize the vec back to a BSON array
        let serialized_vec = mongodb::bson::to_bson(&mapping)?;
        debug!("serialized_vec: {:?}", serialized_vec);

        // Create or update the document
        let update = doc! {
            "$set": { "data": serialized_vec }
        };
        match collection
            .update_one(
                filter,
                update,
                mongodb::options::UpdateOptions::builder()
                    .upsert(true)
                    .build(),
            )
            .await
        {
            Ok(_) => (),
            Err(e) => {
                event!(Level::ERROR, "Failed to set data with error: {e}");
            }
        };
        Ok(())
    }

    async fn get_data<T: DeserializeOwned + Clone>(
        &mut self,
        key: &str,
        value_id: Option<&str>,
    ) -> Result<Option<HashMap<String, T>>, Box<dyn std::error::Error + Send + Sync>> {
        // Tracing
        let span = span!(Level::TRACE, "MongoDbConn::get_data");
        let _enter = span.enter();

        let collection = self
            .client
            .database(&self.index.db_name)
            .collection::<Document>(&self.index.coll_name);

        // Check if the document with the given key exists
        let filter = doc! { "_id": key };
        let doc_find = match collection.find_one(filter.clone(), None).await {
            Ok(doc) => doc,
            Err(e) => {
                event!(Level::ERROR, "Failed to get data with error: {e}");
                return Ok(None);
            }
        };

        if let Some(doc) = doc_find {
            // Deserialize the existing data
            let mapping: HashMap<String, T> =
                mongodb::bson::from_bson(doc.get("data").unwrap().clone())?;

            if let Some(id) = value_id {
                // If value_id is provided, return only the value with the given ID
                if let Some(value) = mapping.get(id) {
                    let mut result: HashMap<String, T> = HashMap::new();
                    result.insert(id.to_string(), value.clone());
                    return Ok(Some(result));
                } else {
                    // Value with the given ID not found
                    event!(Level::ERROR, "Value with ID {id} not found for key {key}");
                    return Ok(None);
                }
            }
            return Ok(Some(mapping));
        }
        event!(Level::ERROR, "Data unsuccessfully deserialized");
        return Ok(None);
    }

    async fn set_data_with_expiry<T: Serialize + std::marker::Send + DeserializeOwned>(
        &mut self,
        key: &str,
        value_id: &str,
        value: T,
        seconds: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Tracing
        let span = span!(Level::TRACE, "MongoDbConn::set_data_with_expiry");
        let _enter = span.enter();

        let collection = self
            .client
            .database(&self.index.db_name)
            .collection::<Document>(&self.index.coll_name);

        // Check if the document with the given key exists
        let filter = doc! { "_id": key };
        let existing_doc = collection.find_one(filter.clone(), None).await?;

        let mut mapping: HashMap<String, T> = if let Some(doc) = existing_doc {
            // Deserialize the existing data
            mongodb::bson::from_bson(doc.get("data").unwrap().clone())?
        } else {
            HashMap::new()
        };

        // Append the new data to the vec
        mapping.insert(value_id.to_string(), value);

        // Serialize the vec back to a BSON array
        let serialized_vec = mongodb::bson::to_bson(&mapping)?;

        // Calculate the expiry time
        let expiry_time = (seconds * 1000) as i64;
        let expiry_bson_datetime = DateTime::from_millis(expiry_time);

        // Create or update the document with the new expiry time
        let update = doc! {
            "$set": {
                "data": serialized_vec,
                "expiry": expiry_bson_datetime,
            }
        };
        collection
            .update_one(
                filter,
                update,
                mongodb::options::UpdateOptions::builder()
                    .upsert(true)
                    .build(),
            )
            .await?;

        trace!("Data set successfully with expiry");

        Ok(())
    }

    async fn del_data(
        &mut self,
        key: &str,
        value_id: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Tracing
        let span = span!(Level::TRACE, "MongoDbConn::del_data");
        let _enter = span.enter();

        let collection = self
            .client
            .database(&self.index.db_name)
            .collection::<Document>(&self.index.coll_name);

        // Build the filter based on the key
        let filter = doc! { "_id": key };

        // If value_id is provided, we need to fetch the document and update it
        if let Some(value_id) = value_id {
            let update = doc! {
                "$unset": {
                    &format!("data.{}", value_id): ""
                }
            };

            match collection.find_one_and_update(filter, update, None).await {
                Ok(result) => {
                    if let Some(_) = result {
                        // Document was found and updated, log success or handle as needed
                        trace!("Data updated successfully");
                    } else {
                        // Document not found
                        event!(Level::ERROR, "Document not found for key: {}", key);
                    }
                }
                Err(e) => {
                    // Handle error from MongoDB
                    event!(Level::ERROR, "Failed to update data with error: {:?}", e);
                    return Err(Box::new(e));
                }
            }
        } else {
            // value_id is None, so delete the entire document
            match collection.delete_one(filter.clone(), None).await {
                Ok(_) => {
                    trace!("Data deleted successfully");
                }
                Err(e) => {
                    event!(Level::ERROR, "Failed to delete data with error: {:?}", e);
                    return Err(Box::new(e));
                }
            };
        }

        Ok(())
    }
}
