use crate::handlers::get_data as get_data_handler;
use crate::handlers::set_data as set_data_handler;
use crate::utils::validate_signature;
use crate::interfaces::InvalidSignature;
use futures::lock::Mutex;
use std::convert::Infallible;
use std::sync::Arc;
use warp::{ Filter, Rejection, Reply };

// Clone component/struct to use in route
pub fn with_node_component<T: Clone + Send>(
    comp: T
) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || comp.clone())
}

/// ========== ROUTES ========== ///

pub fn get_data(
    redis_db: Arc<Mutex<redis::aio::ConnectionManager>>
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("get_data")
        .and(sig_verify_middleware())
        .and(warp::body::json())
        .and(with_node_component(redis_db))
        .and_then(move |_, data, db| get_data_handler(data, db))
        .recover(handle_rejection)
        .with(post_cors())
}

pub fn set_data(
    redis_db: Arc<Mutex<redis::aio::ConnectionManager>>
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("set_data")
        .and(sig_verify_middleware())
        .and(warp::body::json())
        .and(with_node_component(redis_db))
        .and_then(move |_, info, data| set_data_handler(info, data))
        .recover(handle_rejection)
        .with(post_cors())
}

/// ========== UTIL & MIDDLEWARE FUNCTIONS ========== ///

// POST CORS

fn post_cors() -> warp::cors::Builder {
    warp::cors()
        .allow_any_origin()
        .allow_headers(
            vec![
                "Accept",
                "User-Agent",
                "Sec-Fetch-Mode",
                "Referer",
                "Origin",
                "Access-Control-Request-Method",
                "Access-Control-Request-Headers",
                "Access-Control-Allow-Origin",
                "Access-Control-Allow-Headers",
                "Content-Type"
            ]
        )
        .allow_methods(vec!["POST"])
}

// Middleware filter to handle signature verification
pub fn sig_verify_middleware() -> impl Filter<Extract = ((),), Error = Rejection> + Clone {
    warp::path
        ::full()
        .and(warp::header::headers_cloned())
        .and_then(move |_: warp::path::FullPath, headers: warp::hyper::HeaderMap| {
            async move {
                let public_key = headers
                    .get("public_key")
                    .and_then(|n| n.to_str().ok())
                    .unwrap_or_default();

                let address = headers
                    .get("address")
                    .and_then(|n| n.to_str().ok())
                    .unwrap_or_default();

                let signature = headers
                    .get("signature")
                    .and_then(|n| n.to_str().ok())
                    .unwrap_or_default();

                if validate_signature(&public_key, &address, &signature) {
                    // Proceed to the next filter/handler
                    return Ok(());
                }

                // Reject the request with custom rejection
                Err(warp::reject::custom(InvalidSignature))
            }
        })
}

// Rejection handler
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(InvalidSignature) = err.find() {
        // Handle invalid signature error here
        Ok(warp::reply::with_status("Invalid signature", warp::http::StatusCode::BAD_REQUEST))
    } else {
        // For other kinds of rejections, return a generic error
        Ok(
            warp::reply::with_status(
                "Internal Server Error",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            )
        )
    }
}
