// main.rs
pub mod api;
pub mod constants;
pub mod interfaces;
pub mod utils;

#[cfg(test)]
pub mod tests;

use crate::api::routes::*;
use crate::utils::{
    construct_mongodb_conn, construct_redis_conn, init_cuckoo_filter, load_config, print_welcome,
};

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
    // let db_addr = format!(
    //     "{}{}:{}@{}:{}",
    //     config.db_protocol, config.db_user, config.db_password, config.db_url, config.db_port
    // );
    let db_addr = format!("{}{}:{}", config.db_protocol, config.db_url, config.db_port);

    info!("Connecting to Redis at {}", cache_addr);
    info!("Connecting to MongoDB at {}", db_addr);

    let cache_conn = construct_redis_conn(&cache_addr).await;
    let db_conn = construct_mongodb_conn(&db_addr).await;

    let cf_import = match init_cuckoo_filter(db_conn.clone()).await {
        Ok(cf) => cf,
        Err(e) => panic!("Failed to initialize cuckoo filter with error: {}", e),
    };
    let cuckoo_filter = Arc::new(Mutex::new(cf_import));

    info!("Cuckoo filter initialized successfully");

    let routes = get_data(db_conn.clone(), cache_conn.clone(), cuckoo_filter.clone())
        .or(get_data_with_id(
            db_conn.clone(),
            cache_conn.clone(),
            cuckoo_filter.clone(),
        ))
        .or(set_data(
            db_conn.clone(),
            cache_conn.clone(),
            cuckoo_filter.clone(),
            config.body_limit,
            config.cache_ttl,
        ))
        .recover(handle_rejection);

    print_welcome(&db_addr, &cache_addr);

    info!("Server running at localhost:{}", config.extern_port);

    warp::serve(routes)
        .run(([0, 0, 0, 0], config.extern_port))
        .await;
}
