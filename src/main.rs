// main.rs
pub mod api;
pub mod constants;
pub mod interfaces;
pub mod utils;

use crate::api::routes::*;
use crate::utils::{load_config, print_welcome};
use futures::lock::Mutex;
use std::sync::Arc;
use warp::Filter;
use beacon_core::db::handler::KvStoreConnection;
use beacon_core::db::mongo_db::MongoDbConn;
use beacon_core::db::redis_cache::RedisCacheConn;

#[tokio::main]
async fn main() {
    let config = load_config();
    let cache_addr = format!("{}:{}", config.cache_url, config.cache_port);
    let db_addr = format!("{}:{}", config.db_url, config.db_port);
    let cuckoo_filter = Arc::new(Mutex::new(cuckoofilter::CuckooFilter::new()));

    let cache_conn = Arc::new(Mutex::new(RedisCacheConn::init(&cache_addr).await));
    let db_conn = Arc::new(Mutex::new(MongoDbConn::init(&db_addr).await));
    let routes = get_data(
        db_conn.clone(),
        cache_conn.clone(),
        cuckoo_filter.clone(),
        config.body_limit,
    )
    .or(set_data(
        db_conn,
        cache_conn,
        cuckoo_filter,
        config.body_limit,
    ));

    print_welcome(&db_addr, &cache_addr);

    println!("Server running at localhost:{}", config.extern_port);

    warp::serve(routes)
        .run(([0, 0, 0, 0], config.extern_port))
        .await;
}
