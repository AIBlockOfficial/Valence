// main.rs
pub mod crypto;
pub mod db;
pub mod handlers;
pub mod interfaces;
pub mod routes;
pub mod utils;

use crate::db::redis_init;
use crate::routes::*;
use warp::Filter;

#[tokio::main]
async fn main() {
    let redis_conn = redis_init("redis://127.0.0.1").await;
    let routes = get_data(redis_conn.clone()).or(set_data(redis_conn));
    println!("Server running at localhost:3030");

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
