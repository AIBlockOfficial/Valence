use crate::db::handler::KvStoreConnection;
use crate::interfaces::{
    CacheInsertionFailed, CuckooFilterInsertionFailed, CuckooFilterLookupFailed, DBInsertionFailed,
    GetRequestData, InvalidSignature, SetRequestData,
};
use crate::utils::{deserialize_data, serialize_data};
use futures::lock::Mutex;
use std::collections::hash_map::DefaultHasher;
use std::convert::Infallible;
use std::sync::Arc;
use warp::Rejection;

// Implement a custom reject for the error types
impl warp::reject::Reject for InvalidSignature {}
impl warp::reject::Reject for DBInsertionFailed {}
impl warp::reject::Reject for CacheInsertionFailed {}
impl warp::reject::Reject for CuckooFilterInsertionFailed {}
impl warp::reject::Reject for CuckooFilterLookupFailed {}

/// ========= BASE HANDLERS ========= ///

// Route to get data from DB

pub async fn get_data_handler<D: KvStoreConnection, C: KvStoreConnection>(
    db: Arc<Mutex<D>>,
    cache: Arc<Mutex<C>>,
    payload: GetRequestData,
    c_filter: Arc<Mutex<cuckoofilter::CuckooFilter<DefaultHasher>>>,
) -> Result<impl warp::Reply, Rejection> {
    if !c_filter.lock().await.contains(&payload.address) {
        return Err(warp::reject::custom(CuckooFilterLookupFailed));
    }

    let final_data = match cache.lock().await.get_data(&payload.address).await {
        Ok(value) => deserialize_data(value),
        Err(_) => String::from("No data found"),
    };

    Ok(warp::reply::json(&final_data))
}

/// Route to set data (validate the signature)
pub async fn set_data_handler<D: KvStoreConnection, C: KvStoreConnection>(
    payload: SetRequestData,
    db: Arc<Mutex<D>>,
    cache: Arc<Mutex<C>>,
    c_filter: Arc<Mutex<cuckoofilter::CuckooFilter<DefaultHasher>>>,
) -> Result<impl warp::Reply, Rejection> {
    // Add to cache
    let cache_result = cache
        .lock()
        .await
        .set_data(
            &payload.address.clone(),
            Arc::new(Mutex::new(serialize_data(payload.data.clone()))),
        )
        .await;

    // Add to DB
    let db_data = Arc::new(Mutex::new(payload.data));
    let db_result = match cache_result {
        Ok(_) => {
            db.lock()
                .await
                .set_data(&db_config.coll_name, db_data)
                .await
        }
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

// Route to get all currently listed assets
pub async fn get_assets_handler() -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&String::from("Hello, world!")))
}
