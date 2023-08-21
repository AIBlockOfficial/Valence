use crate::db::handler::DbConnection;
use crate::db::mongo_db::{ MongoDbIndex };
use crate::db::redis_cache::{ get_data_from_cache, set_data_in_cache };
use crate::interfaces::{
    DBInsertionFailed,
    GetRequestData,
    InvalidSignature,
    SetRequestData,
    CacheInsertionFailed,
    CuckooFilterInsertionFailed,
    CuckooFilterLookupFailed
};
use crate::utils::{ deserialize_data, serialize_data };
use futures::lock::Mutex;
use std::convert::Infallible;
use std::sync::Arc;
use warp::Rejection;
use std::hash::Hasher;

// Implement a custom reject for the error types
impl warp::reject::Reject for InvalidSignature {}
impl warp::reject::Reject for DBInsertionFailed {}
impl warp::reject::Reject for CacheInsertionFailed {}
impl warp::reject::Reject for CuckooFilterInsertionFailed {}
impl warp::reject::Reject for CuckooFilterLookupFailed {}

/// ========= BASE HANDLERS ========= ///

// Route to get data from DB
pub async fn get_data_handler<T: Default + Hasher>(
    payload: GetRequestData,
    c_filter: Arc<Mutex<cuckoofilter::CuckooFilter<T>>>,
    redis_cache: Arc<Mutex<redis::aio::ConnectionManager>>
) -> Result<impl warp::Reply, Rejection> {
    if !c_filter.lock().await.contains(&payload.address) {
        return Err(warp::reject::custom(CuckooFilterLookupFailed));
    }

    let final_data = match get_data_from_cache(redis_cache, &payload.address).await {
        Ok(value) => deserialize_data(value),
        Err(_) => String::from("No data found"),
    };

    Ok(warp::reply::json(&final_data))
}

// Route to set data (validate the signature)
pub async fn set_data_handler<T: Default + Hasher, D: DbConnection>(
    payload: SetRequestData,
    db: D,
    redis_cache: Arc<Mutex<redis::aio::ConnectionManager>>,
    c_filter: Arc<Mutex<cuckoofilter::CuckooFilter<T>>>,
) -> Result<impl warp::Reply, Rejection> {
    let cache_result = set_data_in_cache(
        redis_cache,
        &payload.address.clone(),
        &serialize_data(payload.data.clone())
    ).await;

    match cache_result {
        Ok(_) =>
            match
                insert_document(
                    &mongo_db,
                    &mongo_config.db_name,
                    &mongo_config.coll_name,
                    payload.data
                ).await
            {
                Ok(_) => Ok(warp::reply::json(&"".to_string())), // TODO: Return a proper response
                Err(_) => Err(warp::reject::custom(DBInsertionFailed)),
            }
        Err(_) => {
            return Err(warp::reject::custom(CacheInsertionFailed));
        }
    }
}

// Route to get all currently listed assets
pub async fn get_assets_handler() -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&String::from("Hello, world!")))
}
