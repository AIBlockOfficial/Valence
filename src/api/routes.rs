use crate::api::handlers::{get_data_handler, set_data_handler};
use crate::db::handler::{CacheHandler, KvStoreConnection};
use futures::lock::Mutex;
use std::sync::Arc;
use tracing::debug;
use valence_core::api::interfaces::CFilterConnection;
use valence_core::api::utils::{
    get_cors, map_api_res, post_cors, sig_verify_middleware, with_node_component,
};
use warp::{Filter, Rejection, Reply};

// ========== BASE ROUTES ========== //

/// GET /get_data_with_id
///
/// Retrieves data associated with a given address and a given id
///
/// ### Arguments
///
/// * `db` - The database connection to use
/// * `cache` - The cache connection to use
/// * `cuckoo_filter` - The cuckoo filter connection to use
pub fn get_data_with_id<
    D: KvStoreConnection + Clone + Send + 'static,
    C: KvStoreConnection + Clone + Send + 'static,
>(
    db: Arc<Mutex<D>>,
    cache: Arc<Mutex<C>>,
    cuckoo_filter: CFilterConnection,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    debug!("Setting up get_data_with_id route");

    warp::path("get_data")
        .and(warp::get())
        .and(sig_verify_middleware())
        .and(warp::header::headers_cloned())
        .and(warp::path::param::<String>())
        .and(with_node_component(cache))
        .and(with_node_component(db))
        .and(with_node_component(cuckoo_filter))
        .and_then(move |_, headers, value_id: String, cache, db, cf| {
            // Add type annotation for headers parameter
            debug!("GET_DATA requested");
            map_api_res(get_data_handler(headers, Some(value_id), db, cache, cf))
        })
        .with(get_cors())
}

pub fn get_data<
    D: KvStoreConnection + Clone + Send + 'static,
    C: KvStoreConnection + Clone + Send + 'static,
>(
    db: Arc<Mutex<D>>,
    cache: Arc<Mutex<C>>,
    cuckoo_filter: CFilterConnection,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    debug!("Setting up get_data route");

    warp::path("get_data")
        .and(warp::get())
        .and(sig_verify_middleware())
        .and(warp::header::headers_cloned())
        .and(with_node_component(cache))
        .and(with_node_component(db))
        .and(with_node_component(cuckoo_filter))
        .and_then(move |_, headers, cache, db, cf| {
            // Add type annotation for headers parameter
            debug!("GET_DATA requested");
            map_api_res(get_data_handler(headers, None, db, cache, cf))
        })
        .with(get_cors())
}

/// POST /set_data
///
/// Sets data for a given address
///
/// ### Arguments
///
/// * `db` - The database connection to use
/// * `cache` - The cache connection to use
/// * `cuckoo_filter` - The cuckoo filter connection to use
/// * `body_limit` - The maximum size of the request body
pub fn set_data<
    D: KvStoreConnection + Clone + Send + Sync + 'static,
    C: KvStoreConnection + CacheHandler + Clone + Send + Sync + 'static,
>(
    db: Arc<Mutex<D>>,
    cache: Arc<Mutex<C>>,
    cuckoo_filter: CFilterConnection,
    body_limit: u64,
    cache_ttl: usize,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    debug!("Setting up set_data route");

    warp::path("set_data")
        .and(warp::post())
        .and(sig_verify_middleware())
        .and(warp::body::content_length_limit(body_limit))
        .and(warp::body::json())
        .and(with_node_component(cache))
        .and(with_node_component(db))
        .and(with_node_component(cuckoo_filter))
        .and(with_node_component(cache_ttl))
        .and_then(move |_, info, cache, db, cf, cttl| {
            debug!("SET_DATA requested");
            map_api_res(set_data_handler(info, db, cache, cf, cttl))
        })
        .with(post_cors())
}
