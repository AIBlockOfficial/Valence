use warp::Rejection;
use std::convert::Infallible;
use crate::utils::validate_signature;
use crate::interfaces::{InvalidSignature, GetRequestData};

// Implement a custom reject for the InvalidSignature error
impl warp::reject::Reject for InvalidSignature {}

// Route to get data (dummy data for demonstration)
pub async fn get_data() -> Result<impl warp::Reply, Infallible> {
    let data = GetRequestData {
        public_key: "public_key".to_string(),
        address: "address".to_string(),
        signature: "signature".to_string(),
    };
    Ok(warp::reply::json(&data))
}

// Route to set data (validate the signature)
pub async fn set_data(data: GetRequestData) -> Result<impl warp::Reply, Rejection> {
    // Validate the signature
    if validate_signature(&data.public_key, &data.address, &data.signature) {
        // Do something with the data (e.g., store it in a database)
        // For this example, we simply print the data
        println!("Received data: {:?}", data);

        Ok(warp::reply::json(&data))
    } else {
        // Return an error if the signature is invalid
        Err(warp::reject::custom(InvalidSignature))
    }
}