// main.rs

use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};

pub mod utils;
pub mod crypto;
pub mod handlers;
pub mod interfaces;

use crate::interfaces::InvalidSignature;
use crate::handlers::{get_data, set_data};

#[tokio::main]
async fn main() {
    let get_route = warp::path!("get_data").and_then(get_data);
    let set_route = warp::path!("set_data")
        .and(warp::body::json())
        .and_then(set_data)
        .recover(handle_rejection);

    let routes = get_route.or(set_route);

    println!("Server running at localhost:3030");

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

// Function to handle custom rejections
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(InvalidSignature) = err.find() {
        // Handle invalid signature error here
        Ok(warp::reply::with_status(
            "Invalid signature",
            warp::http::StatusCode::BAD_REQUEST,
        ))
    } else {
        // For other kinds of rejections, return a generic error
        Ok(warp::reply::with_status(
            "Internal Server Error",
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}
