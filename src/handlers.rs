use std::sync::Arc;
use futures::lock::Mutex;
use warp::Rejection;
use std::convert::Infallible;
use crate::db::{redis_set_data, redis_get_data};
use crate::utils::validate_signature;
use crate::interfaces::{InvalidSignature, GetRequestData, DBInsertionFailed};

// Implement a custom reject for the error types
impl warp::reject::Reject for InvalidSignature {}
impl warp::reject::Reject for DBInsertionFailed {}

// Route to get data from DB
pub async fn get_data(data: GetRequestData, redis_db: Arc<Mutex<redis::aio::ConnectionManager>>) -> Result<impl warp::Reply, Infallible> {
    let final_data = match redis_get_data(redis_db, &data.address).await {
        Ok(value) => value,
        Err(_) => String::from("No data found"),
    };

    Ok(warp::reply::json(&final_data))
}

// Route to set data (validate the signature)
pub async fn set_data(data: GetRequestData, redis_db: Arc<Mutex<redis::aio::ConnectionManager>>) -> Result<impl warp::Reply, Rejection> {
    // Validate the signature
    if validate_signature(&data.public_key, &data.address, &data.signature) {
        println!("Received data: {:?}", data);
        
        match redis_set_data(redis_db, &data.public_key, &data).await {
            Ok(value) => Ok(warp::reply::json(&data)),
            Err(_) => Err(warp::reject::custom(DBInsertionFailed)),
        }
        
    } else {
        // Return an error if the signature is invalid
        Err(warp::reject::custom(InvalidSignature))
    }
}