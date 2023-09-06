use crate::interfaces::{GetRequestData, SetRequestData};
use weaver_core::api::errors::ApiErrorType;
use weaver_core::db::handler::KvStoreConnection;
use weaver_core::utils::{deserialize_data, serialize_data};
use weaver_core::api::responses::{json_serialize_embed, CallResponse, JsonReply};
use weaver_core::api::interfaces::{CFilterConnection, CacheConnection, DbConnection};

/// ========= BASE HANDLERS ========= ///

/// Route to get data from DB
///
/// ### Arguments
///
/// * `payload` - Request payload
/// * `db` - Database connection
/// * `cache` - Cache connection
/// * `c_filter` - Cuckoo filter connection
pub async fn get_data_handler(
    payload: GetRequestData,
    db: DbConnection,
    cache: CacheConnection,
    c_filter: CFilterConnection,
) -> Result<JsonReply, JsonReply> {
    let r = CallResponse::new("get_data");

    // Check if address is in cuckoo filter
    if !c_filter.lock().await.contains(&payload.address) {
        return r.into_err_internal(ApiErrorType::CuckooFilterLookupFailed);
    }

    // Check cache first
    let cache_result = cache.lock().await.get_data(&payload.address).await;

    match cache_result {
        Ok(value) => {
            // Return data from cache
            let final_data = deserialize_data::<String>(value.unwrap());
            r.into_ok(
                "Data retrieved successfully",
                json_serialize_embed(final_data),
            )
        }
        Err(_) => {
            // Get data from DB
            let db_result = db.lock().await.get_data(&payload.address).await;

            match db_result {
                Ok(value) => {
                    // Return data from DB
                    let final_data = deserialize_data::<String>(value.unwrap());
                    r.into_ok(
                        "Data retrieved successfully",
                        json_serialize_embed(final_data),
                    )
                }
                Err(_) => r.into_err_internal(ApiErrorType::DBInsertionFailed),
            }
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
pub async fn set_data_handler(
    payload: SetRequestData,
    db: DbConnection,
    cache: CacheConnection,
    c_filter: CFilterConnection,
) -> Result<JsonReply, JsonReply> {
    let r = CallResponse::new("set_data");

    // Add to cache
    let cache_result = cache
        .lock()
        .await
        .set_data(&payload.address.clone(), serialize_data(&payload.data))
        .await;

    // Add to DB
    let db_result = match cache_result {
        Ok(_) => {
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
            "Data retrieved succcessfully",
            json_serialize_embed(payload.address),
        ),
        Err(_) => r.into_err_internal(ApiErrorType::CuckooFilterInsertionFailed),
    }
}
