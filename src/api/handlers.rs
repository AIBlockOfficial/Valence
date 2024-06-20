use crate::api::utils::retrieve_from_db;
use crate::interfaces::SetRequestData;
use futures::lock::Mutex;
use serde_json::Value;
use std::sync::Arc;
use tracing::{error, info, warn};
use valence_core::api::errors::ApiErrorType;
use valence_core::api::interfaces::CFilterConnection;
use valence_core::api::responses::{json_serialize_embed, CallResponse, JsonReply};
use valence_core::db::handler::{CacheHandler, KvStoreConnection};
use valence_core::utils::serialize_data;

// ========= BASE HANDLERS ========= //

/// Route to get data from DB
///
/// ### Arguments
///
/// * `payload` - Request payload
/// * `db` - Database connection
/// * `cache` - Cache connection
/// * `c_filter` - Cuckoo filter connection
pub async fn get_data_handler<
    D: KvStoreConnection + Clone + Send + 'static,
    C: KvStoreConnection + Clone + Send + 'static,
>(
    headers: warp::hyper::HeaderMap,
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
        error!("Address not found in cuckoo filter");
        return r.into_err_internal(ApiErrorType::CuckooFilterLookupFailed);
    }

    // Check cache first
    let mut cache_lock_result = cache.lock().await;
    let cache_result: Result<Option<Vec<String>>, _> =
        cache_lock_result.get_data::<String>(address).await;

    info!("Cache result: {:?}", cache_result);

    match cache_result {
        Ok(value) => {
            match value {
                Some(value) => {
                    info!("Data retrieved from cache");
                    let data = value
                        .iter()
                        .map(|v| serde_json::from_str(v).unwrap())
                        .collect::<Vec<Value>>();
                    return r.into_ok("Data retrieved successfully", json_serialize_embed(data));
                }
                None => {
                    // Default to checking from DB if cache is empty
                    warn!("Cache lookup failed for address: {}", address);
                    retrieve_from_db(db, address).await
                }
            }
        }
        Err(_) => {
            warn!("Cache lookup failed for address: {}", address);
            warn!("Attempting to retrieve data from DB");

            // Get data from DB
            retrieve_from_db(db, address).await
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

    // Add to cache
    let cache_result = cache
        .lock()
        .await
        .set_data(&payload.address.clone(), serialize_data(&payload.data))
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
                .set_data(&payload.address, payload.data)
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
