// main.rs
pub mod utils;
pub mod crypto;
pub mod routes;
pub mod handlers;
pub mod interfaces;

use warp::Filter;
use crate::routes::{get_data, set_data};

#[tokio::main]
async fn main() {
    let routes = get_data().or(set_data());
    println!("Server running at localhost:3030");

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
