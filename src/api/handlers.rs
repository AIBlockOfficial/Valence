use crate::api::interfaces::{ CacheConnection, CFilterConnection, DbConnection };
use crate::interfaces::{
    CacheInsertionFailed,
    CuckooFilterInsertionFailed,
    CuckooFilterLookupFailed,
    DBInsertionFailed,
    GetRequestData,
    InvalidSignature,
    SetRequestData,
};
use crate::utils::{ deserialize_data, serialize_data };
use crate::db::handler::KvStoreConnection;
use warp::Rejection;

/// ========= ERROR REJECT IMPL ========= ///

impl warp::reject::Reject for InvalidSignature {}
impl warp::reject::Reject for DBInsertionFailed {}
impl warp::reject::Reject for CacheInsertionFailed {}
impl warp::reject::Reject for CuckooFilterInsertionFailed {}
impl warp::reject::Reject for CuckooFilterLookupFailed {}

/// ========= BASE HANDLERS ========= ///

/// Route to get data from DB
pub async fn get_data_handler(
    db: DbConnection,
    cache: CacheConnection,
    payload: GetRequestData,
    c_filter: CFilterConnection
) -> Result<impl warp::Reply, Rejection> {
    // Check if address is in cuckoo filter
    if !c_filter.lock().await.contains(&payload.address) {
        return Err(warp::reject::custom(CuckooFilterLookupFailed));
    }

    // Check cache first
    let cache_result = cache.lock().await.get_data(&payload.address).await;

    match cache_result {
        Ok(value) => {
            // Return data from cache
            let final_data = deserialize_data::<String>(value.unwrap());
            Ok(warp::reply::json(&final_data))
        }
        Err(_) => {
            // Get data from DB
            let db_result = db.lock().await.get_data(&payload.address).await;

            match db_result {
                Ok(value) => {
                    // Return data from DB
                    let final_data = deserialize_data::<String>(value.unwrap());
                    Ok(warp::reply::json(&final_data))
                }
                Err(_) => { Err(warp::reject::custom(DBInsertionFailed)) }
            }
        }
    }
}

/// Route to set data (validate the signature)
pub async fn set_data_handler(
    payload: SetRequestData,
    db: DbConnection,
    db_key: String,
    cache: CacheConnection,
    c_filter: CFilterConnection
) -> Result<impl warp::Reply, Rejection> {
    // Add to cache
    let cache_result = cache
        .lock().await
        .set_data(&payload.address.clone(), serialize_data(&payload.data)).await;

    // Add to DB
    let db_result = match cache_result {
        Ok(_) => { db.lock().await.set_data(&db_key, payload.data).await }
        Err(_) => {
            return Err(warp::reject::custom(CacheInsertionFailed));
        }
    };

    // Add to cuckoo filter
    let c_filter_result = match db_result {
        Ok(_) => c_filter.lock().await.add(&payload.address),
        Err(_) => {
            return Err(warp::reject::custom(DBInsertionFailed));
        }
    };

    match c_filter_result {
        Ok(_) => Ok(warp::reply::json(&String::from("Data added successfully"))),
        Err(_) => Err(warp::reject::custom(CuckooFilterInsertionFailed)),
    }
}
