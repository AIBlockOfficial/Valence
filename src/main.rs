// main.rs
pub mod api;
pub mod constants;
pub mod interfaces;
pub mod utils;

#[cfg(test)]
pub mod tests;

use crate::api::routes::*;
use crate::utils::{construct_mongodb_conn, construct_redis_conn, load_config, print_welcome};
use futures::lock::Mutex;
use std::sync::Arc;
use tracing::info;
use valence_core::api::utils::handle_rejection;

use warp::Filter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = load_config();
    let cache_addr = format!("{}:{}", config.cache_url, config.cache_port);
    let db_addr = format!(
        "{}{}:{}@{}:{}",
        config.db_protocol, config.db_user, config.db_password, config.db_url, config.db_port
    );
    let cuckoo_filter = Arc::new(Mutex::new(cuckoofilter::CuckooFilter::new()));

    info!("Connecting to Redis at {}", cache_addr);
    info!("Connecting to MongoDB at {}", db_addr);

    let cache_conn = construct_redis_conn(&cache_addr).await;
    let db_conn = construct_mongodb_conn(&db_addr).await;

    let routes = get_data(db_conn.clone(), cache_conn.clone(), cuckoo_filter.clone())
        .or(set_data(
            db_conn.clone(),
            cache_conn.clone(),
            cuckoo_filter.clone(),
            config.body_limit,
            config.cache_ttl,
        ))
        // .or(listings(market_db_conn.clone(), cache_conn.clone()))
        // .or(orders_by_id(
        //     market_db_conn.clone(),
        //     cache_conn.clone(),
        //     cuckoo_filter.clone(),
        // ))
        // .or(orders_send(
        //     market_db_conn.clone(),
        //     cache_conn.clone(),
        //     cuckoo_filter.clone(),
        //     config.body_limit,
        // ))
        // .or(listing_send(
        //     market_db_conn.clone(),
        //     cache_conn.clone(),
        //     cuckoo_filter.clone(),
        //     config.body_limit,
        // ))
        .recover(handle_rejection);

    print_welcome(&db_addr, &cache_addr);

    info!("Server running at localhost:{}", config.extern_port);

    warp::serve(routes)
        .run(([0, 0, 0, 0], config.extern_port))
        .await;
}
