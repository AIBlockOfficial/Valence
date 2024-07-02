use crate::db::handler::KvStoreConnection;
use futures::lock::Mutex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use valence_core::api::errors::ApiErrorType;
use valence_core::api::responses::{json_serialize_embed, CallResponse, JsonReply};

/// Retrieve data from the database
///
/// ### Arguments
///
/// * `db` - Database connection
/// * `address` - Address to retrieve data from
/// * `value_id` - Value ID to retrieve (Optional, if not provided, all values for the address are retrieved)
pub async fn retrieve_from_db<D: KvStoreConnection + Clone + Send + 'static>(
    db: Arc<Mutex<D>>,
    address: &str,
    value_id: Option<&str>,
) -> Result<JsonReply, JsonReply> {
    let r = CallResponse::new("get_data");
    info!("RETRIEVE_FROM_DB requested with address: {:?}", address);

    let db_result: Result<Option<HashMap<String, Value>>, _> =
        db.lock().await.get_data(address, value_id).await;

    match db_result {
        Ok(data) => match data {
            Some(value) => {
                return r.into_ok("Data retrieved successfully", json_serialize_embed(value));
            }
            None => {
                info!("Data not found in DB");
                return r.into_err_internal(ApiErrorType::Generic("Data not found".to_string()));
            }
        },
        Err(_) => r.into_err_internal(ApiErrorType::Generic(
            "Full Valence chain retrieval failed".to_string(),
        )),
    }
}

/// Deletes data from the database
///
/// ### Arguments
///
/// * `db` - Database connection
/// * `address` - Address to retrieve data from
/// * `value_id` - Value ID to delete (Optional, if not provided, all values for the address are deleted)
pub async fn delete_from_db<D: KvStoreConnection + Clone + Send + 'static>(
    db: Arc<Mutex<D>>,
    address: &str,
    value_id: Option<&str>,
) -> Result<JsonReply, JsonReply> {
    let r = CallResponse::new("del_data");
    info!("DELETE_FROM_DB requested with address: {:?}", address);

    let db_result = db.lock().await.del_data(address, value_id).await;

    match db_result {
        Ok(_) => r.into_ok("Data deleted successfully", json_serialize_embed(address)),
        Err(_) => r.into_err_internal(ApiErrorType::Generic(format!(
            "Failed to delete DB entry for {:?}",
            address
        ))),
    }
}

/// Serialize all entries in a HashMap
/// 
/// ### Arguments
///
/// * `data` - HashMap of key-value pairs to serialize
pub fn serialize_all_entries(data: HashMap<String, String>) -> HashMap<String, Value> {
    let mut output = HashMap::new();

    for (key, value) in data {
        match serde_json::from_str(&value) {
            Ok(json_value) => {
                output.insert(key, json_value);
            }
            Err(_e) => {
                output.insert(key, json!(value));
            }
        }
    }
    output
}
