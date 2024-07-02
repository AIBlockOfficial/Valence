use crate::api::utils::{retrieve_from_db, delete_from_db, serialize_all_entries};
use crate::interfaces::{SetRequestData, SetSaveData};
use crate::db::handler::{CacheHandler, KvStoreConnection};
use futures::lock::Mutex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, debug};
use valence_core::api::errors::ApiErrorType;
use valence_core::api::interfaces::CFilterConnection;
use valence_core::api::responses::{json_serialize_embed, CallResponse, JsonReply};
use valence_core::utils::serialize_data;

// ========= BASE HANDLERS ========= //

/// Route to get data from DB
///
/// ### Arguments
///
/// * `headers` - Request headers
/// * `value_id` - Value ID to retrieve (Optional, if not provided, all values for the address are retrieved)
/// * `db` - Database connection
/// * `cache` - Cache connection
/// * `c_filter` - Cuckoo filter connection
pub async fn get_data_handler<
    D: KvStoreConnection + Clone + Send + 'static,
    C: KvStoreConnection + Clone + Send + 'static,
>(
    headers: warp::hyper::HeaderMap,
    value_id: Option<String>,
    db: Arc<Mutex<D>>,
    cache: Arc<Mutex<C>>,
    c_filter: CFilterConnection,
) -> Result<JsonReply, JsonReply> {
    let r = CallResponse::new("get_data");
    info!("GET_DATA requested with headers: {:?}", headers);

    let address = headers
        .get("address")
        .and_then(|n| n.to_str().ok())
        .unwrap_or_default();

    // Check if address is in cuckoo filter
    if !c_filter.lock().await.contains(&address) {
        error!("{}", ApiErrorType::CuckooFilterLookupFailed );
        return r.into_err_internal(ApiErrorType::CuckooFilterLookupFailed);
    }

    // Check cache first
    let mut cache_lock_result = cache.lock().await;
    let cache_result: Result<Option<HashMap<String, String>>, _> = cache_lock_result
        .get_data::<String>(address, value_id.as_deref())
        .await;

    debug!("Cache result: {:?}", cache_result);

    match cache_result {
        Ok(value) => {
            match value {
                Some(value) => {
                    info!("Data retrieved from cache");
                    if let Some(id) = value_id {
                        if !value.contains_key(&id) {
                            return r.into_err_internal(ApiErrorType::Generic(
                                "Value ID not found".to_string(),
                            ));
                        }

                        let data = value.get(&id).unwrap().clone();
                        let final_result: Value = serde_json::from_str(&data).unwrap();
                        return r.into_ok(
                            "Data retrieved successfully",
                            json_serialize_embed(final_result),
                        );
                    }

                    let final_value = serialize_all_entries(value);

                    return r.into_ok(
                        "Data retrieved successfully",
                        json_serialize_embed(final_value),
                    );
                }
                None => {
                    // Default to checking from DB if cache is empty
                    debug!("Cache lookup failed for address: {}", address);
                    retrieve_from_db(db, address, value_id.as_deref()).await
                }
            }
        }
        Err(_) => {
            debug!("Attempting to retrieve data from DB");
            // Get data from DB
            retrieve_from_db(db, address, value_id.as_deref()).await
        }
    }
}

/// Route to set data
///
/// ### Arguments
///
/// * `payload` - Request payload
/// * `db` - Database connection
/// * `cache` - Cache connection
/// * `c_filter` - Cuckoo filter connection
/// * `cache_ttl` - Cache TTL
pub async fn set_data_handler<
    D: KvStoreConnection + Clone + Send + 'static,
    C: KvStoreConnection + CacheHandler + Clone + Send + 'static,
>(
    payload: SetRequestData,
    db: Arc<Mutex<D>>,
    cache: Arc<Mutex<C>>,
    c_filter: CFilterConnection,
    cache_ttl: usize,
) -> Result<JsonReply, JsonReply> {
    let r = CallResponse::new("set_data");
    info!("SET_DATA requested with payload: {:?}", payload);

    let data_to_save: SetSaveData = {
        SetSaveData {
            address: payload.address.clone(),
            data: payload.data.clone(),
        }
    };

    // Add to cache
    let cache_result = cache
        .lock()
        .await
        .set_data(
            &payload.address.clone(),
            &payload.data_id,
            serialize_data(&data_to_save),
        )
        .await;

    // Add to DB
    let db_result = match cache_result {
        Ok(_) => {
            // Set key expiry
            let _ = cache
                .lock()
                .await
                .expire_entry(&payload.address, cache_ttl)
                .await;

            db.lock()
                .await
                .set_data(&payload.address, &payload.data_id, data_to_save)
                .await
        }
        Err(_) => {
            return r.into_err_internal(ApiErrorType::CacheInsertionFailed);
        }
    };

    // Add to cuckoo filter
    let c_filter_result = match db_result {
        Ok(_) => c_filter.lock().await.add(&payload.address),
        Err(_) => {
            return r.into_err_internal(ApiErrorType::DBInsertionFailed);
        }
    };

    match c_filter_result {
        Ok(_) => r.into_ok(
            "Data set successfully",
            json_serialize_embed(payload.address),
        ),
        Err(_) => r.into_err_internal(ApiErrorType::CuckooFilterInsertionFailed),
    }
}

/// Route to del data from DB
///
/// /// ### Arguments
///
/// * `headers` - Request headers
/// * `value_id` - Value ID to retrieve (Optional, if not provided, all values for the address are deleted)
/// * `db` - Database connection
/// * `cache` - Cache connection
/// * `c_filter` - Cuckoo filter connection
pub async fn del_data_handler<
    D: KvStoreConnection + Clone + Send + 'static,
    C: KvStoreConnection + Clone + Send + 'static,
>(
    headers: warp::hyper::HeaderMap,
    value_id: Option<String>,
    db: Arc<Mutex<D>>,
    cache: Arc<Mutex<C>>,
    c_filter: CFilterConnection,
) -> Result<JsonReply, JsonReply> {
    let r = CallResponse::new("del_data");
    info!("DEL_DATA requested with headers: {:?}", headers);

    let address = headers
        .get("address")
        .and_then(|n| n.to_str().ok())
        .unwrap_or_default();

    // delete address in cuckoo filter if no value_id is provided
    if value_id.is_none() && !c_filter.lock().await.delete(&address) {
        error!("Address not found in cuckoo filter");
        return r.into_err_internal(ApiErrorType::CuckooFilterLookupFailed);
    }

    debug!("delete on cuckoo filter successful");

    // Check cache
    let mut cache_lock_result = cache.lock().await;
    let cache_result = cache_lock_result
        .del_data(address, value_id.as_deref())
        .await;

    match cache_result {
        Ok(_) => {
            debug!("Data deleted from cache");
            return delete_from_db(db, address, value_id.as_deref()).await
        }
        Err(_) => {
            error!("Cache deletion failed for address: {}", address);
            return r.into_err_internal(ApiErrorType::Generic("Cache deletion failed".to_string()));
        }
    }
}
