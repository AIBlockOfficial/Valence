// main.rs
pub mod db;
pub mod utils;
pub mod crypto;
pub mod routes;
pub mod handlers;
pub mod interfaces;

use warp::Filter;
use crate::db::redis_init;
use crate::routes::{get_data, set_data};

#[tokio::main]
async fn main() {
    let redis_conn = redis_init("redis://127.0.0.1").await;
    let routes = get_data(redis_conn.clone()).or(set_data(redis_conn));
    println!("Server running at localhost:3030");

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
