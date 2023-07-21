use std::sync::Arc;
use futures::lock::Mutex;
use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};
use crate::interfaces::InvalidSignature;
use crate::handlers::set_data as set_data_handler;
use crate::handlers::get_data as get_data_handler;

// Clone component/struct to use in route
pub fn with_node_component<T: Clone + Send>(
    comp: T,
) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || comp.clone())
}

/// ========== ROUTES ========== ///

pub fn get_data(redis_db: Arc<Mutex<redis::aio::ConnectionManager>>) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("get_data")
        .and(warp::body::json())
        .and(with_node_component(redis_db))
        .and_then(move |info, data| get_data_handler(info, data))
        .recover(handle_rejection)
        .with(post_cors())
}

pub fn set_data(redis_db: Arc<Mutex<redis::aio::ConnectionManager>>) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("set_data")
        .and(warp::body::json())
        .and(with_node_component(redis_db))
        .and_then(|info, data| set_data_handler(info, data))
        .recover(handle_rejection)
        .with(post_cors())
}


/// ========== UTIL FUNCTIONS ========== ///

// GET CORS
pub fn get_cors() -> warp::cors::Builder {
    warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Accept",
            "User-Agent",
            "Sec-Fetch-Mode",
            "Referer",
            "Origin",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "Access-Control-Allow-Origin",
            "Access-Control-Allow-Headers",
            "Content-Type",
        ])
        .allow_methods(vec!["GET"])
}

// POST CORS
pub fn post_cors() -> warp::cors::Builder {
    warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Accept",
            "User-Agent",
            "Sec-Fetch-Mode",
            "Referer",
            "Origin",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "Access-Control-Allow-Origin",
            "Access-Control-Allow-Headers",
            "Content-Type",
        ])
        .allow_methods(vec!["POST"])
}

// Rejection handler
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