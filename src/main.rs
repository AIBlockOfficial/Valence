// main.rs
pub mod crypto;
pub mod db;
pub mod handlers;
pub mod interfaces;
pub mod routes;
pub mod utils;
pub mod constants;
pub mod market;

use crate::db::redis_cache::init_cache;
use crate::routes::*;
use crate::utils::load_config;
use warp::Filter;

#[tokio::main]
async fn main() {
    let config = load_config();
    let db_addr = format!("{}:{}",
        config.db_url,
        config.db_port
    );

    let cache_conn = init_cache(&db_addr).await;
    let routes = get_data(cache_conn.clone()).or(set_data(cache_conn));
    println!("Server running at localhost:3030");

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
