// main.rs
pub mod constants;
pub mod crypto;
pub mod db;
pub mod interfaces;
pub mod api;
pub mod utils;

use crate::db::handler::KvStoreConnection;
use crate::db::mongo_db::MongoDbConn;
use crate::db::redis_cache::RedisCacheConn;
use crate::api::routes::*;
use crate::utils::load_config;
use futures::lock::Mutex;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = load_config();
    let cache_addr = format!("{}:{}", config.cache_url, config.cache_port);
    let db_addr = format!("{}:{}", config.db_url, config.db_port);
    let cuckoo_filter = Arc::new(Mutex::new(cuckoofilter::CuckooFilter::new()));

    let cache_conn = Arc::new(Mutex::new(RedisCacheConn::init(&cache_addr).await));
    let db_conn = Arc::new(Mutex::new(MongoDbConn::init(&db_addr).await));
    let routes = get_data(db_conn, cache_conn.clone(), cuckoo_filter);
    println!("Server running at localhost:3030");

    warp::serve(routes).run(([127, 0, 0, 1], config.extern_port)).await;
}
