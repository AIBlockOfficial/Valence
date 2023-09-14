// main.rs
pub mod api;
pub mod constants;
pub mod interfaces;
pub mod utils;

use crate::api::routes::*;
use crate::utils::{ load_config, print_welcome };
use weaver_core::api::utils::handle_rejection;
use weaver_core::db::handler::KvStoreConnection;
use weaver_core::db::mongo_db::MongoDbConn;
use weaver_core::db::redis_cache::RedisCacheConn;
use futures::lock::Mutex;
use std::sync::Arc;
use warp::Filter;
use weaver_market::api::routes::*;
use weaver_market::db::interfaces::MongoDbConnWithMarket;

#[tokio::main]
async fn main() {
    let config = load_config();
    let cache_addr = format!("{}:{}", config.cache_url, config.cache_port);
    let db_addr = format!("{}:{}", config.db_url, config.db_port);
    let cuckoo_filter = Arc::new(Mutex::new(cuckoofilter::CuckooFilter::new()));

    let cache_conn = Arc::new(Mutex::new(RedisCacheConn::init(&cache_addr).await));
    let db_conn = Arc::new(Mutex::new(MongoDbConn::init(&db_addr).await));
    let market_db_conn = MongoDbConnWithMarket::new(db_conn.clone());

    let routes = get_data(
        db_conn.clone(),
        cache_conn.clone(),
        cuckoo_filter.clone(),
        config.body_limit
    )
        .or(set_data(db_conn.clone(), cache_conn.clone(), cuckoo_filter.clone(), config.body_limit))
        .or(listings(market_db_conn.clone(), cache_conn.clone()))
        .or(orders_by_id(market_db_conn.clone(), cache_conn.clone(), cuckoo_filter.clone()))
        .or(orders_send(market_db_conn.clone(), cache_conn.clone(), cuckoo_filter.clone(), config.body_limit))
        .or(
            listing_send(
                market_db_conn.clone(),
                cache_conn.clone(),
                cuckoo_filter.clone(),
                config.body_limit
            )
        )
        .recover(handle_rejection);

    print_welcome(&db_addr, &cache_addr);

    println!("Server running at localhost:{}", config.extern_port);

    warp::serve(routes).run(([0, 0, 0, 0], config.extern_port)).await;
}
