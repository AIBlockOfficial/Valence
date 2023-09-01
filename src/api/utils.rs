use crate::api::errors::ApiErrorType;
use crate::api::responses::JsonReply;
use crate::utils::validate_signature;
use std::convert::Infallible;
use warp::{Filter, Future, Rejection, Reply};

impl warp::reject::Reject for ApiErrorType {}

/// Clone component/struct to use in route
///
/// ### Arguments
///
/// * `comp` - Component/struct to clone
pub fn with_node_component<T: Clone + Send>(
    comp: T,
) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || comp.clone())
}

/// Easy and simple POST CORS
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

/// Middleware filter to handle signature verification
pub fn sig_verify_middleware() -> impl Filter<Extract = ((),), Error = Rejection> + Clone {
    warp::path::full()
        .and(warp::header::headers_cloned())
        .and_then(
            move |_: warp::path::FullPath, headers: warp::hyper::HeaderMap| {
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

                    if validate_signature(public_key, address, signature) {
                        // Proceed to the next filter/handler
                        println!("Signature verified");
                        return Ok(());
                    }

                    Err(warp::reject::custom(ApiErrorType::InvalidSignature))
                }
            },
        )
}

/// Rejection handler
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(ApiErrorType::InvalidSignature) = err.find() {
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

/// Map API result to warp reply
pub fn map_api_res(
    r: impl Future<Output = Result<JsonReply, JsonReply>>,
) -> impl Future<Output = Result<impl warp::Reply, warp::Rejection>> {
    use futures::future::TryFutureExt;
    r.map_ok_or_else(Ok, Ok)
}
