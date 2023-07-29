use crate::db::{ redis_get_data, redis_set_data };
use crate::interfaces::{ DBInsertionFailed, GetRequestData, SetRequestData, InvalidSignature };
use crate::utils::{ deserialize_data, serialize_data };
use futures::lock::Mutex;
use std::convert::Infallible;
use std::sync::Arc;
use warp::Rejection;

// Implement a custom reject for the error types
impl warp::reject::Reject for InvalidSignature {}
impl warp::reject::Reject for DBInsertionFailed {}

// Route to get data from DB
pub async fn get_data(
    payload: GetRequestData,
    redis_db: Arc<Mutex<redis::aio::ConnectionManager>>
) -> Result<impl warp::Reply, Infallible> {
    let final_data = match redis_get_data(redis_db, &payload.address).await {
        Ok(value) => deserialize_data(value),
        Err(_) => String::from("No data found"),
    };

    Ok(warp::reply::json(&final_data))
}

// Route to set data (validate the signature)
pub async fn set_data(
    payload: SetRequestData,
    redis_db: Arc<Mutex<redis::aio::ConnectionManager>>
) -> Result<impl warp::Reply, Rejection> {
    match redis_set_data(redis_db, &payload.address.clone(), &serialize_data(payload.data.clone())).await {
        Ok(_) => Ok(warp::reply::json(&payload.data)),
        Err(_) => Err(warp::reject::custom(DBInsertionFailed)),
    }
}