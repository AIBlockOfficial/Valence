pub mod constants;
pub mod interfaces;

use crate::api::routes;
use crate::tests::constants::{TEST_VALID_ADDRESS, TEST_VALID_PUB_KEY, TEST_VALID_SIG};
use crate::tests::interfaces::DbStub;
use futures::lock::Mutex;
use std::sync::Arc;
use valence_core::api::utils::handle_rejection;
use valence_core::db::handler::KvStoreConnection;
use warp::Filter;

//========== TESTS ==========//

#[tokio::test(flavor = "current_thread")]
async fn test_get_data_empty() {
    //
    // Arrange
    //
    let request = warp::test::request()
        .method("GET")
        .header("public_key", TEST_VALID_PUB_KEY)
        .header("address", TEST_VALID_ADDRESS)
        .header("signature", TEST_VALID_SIG)
        .path("/get_data");

    let db_stub = Arc::new(Mutex::new(DbStub::init("").await.unwrap()));
    let cache_stub = Arc::new(Mutex::new(DbStub::init("").await.unwrap()));
    let cfilter = Arc::new(Mutex::new(cuckoofilter::CuckooFilter::new()));

    //
    // Act
    //
    let filter = routes::get_data(db_stub, cache_stub, cfilter).recover(handle_rejection);
    let res = request.reply(&filter).await;

    //
    // Assert
    //
    assert_eq!(res.status(), 500);
    assert_eq!(
        res.body(),
        "{\"status\":\"Error\",\"reason\":\"Cuckoo filter lookup failed, data for address not found on this Valence\",\"route\":\"get_data\",\"content\":\"null\"}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_data() {
    //
    // Arrange
    //
    let request = warp::test::request()
        .method("GET")
        .header("public_key", TEST_VALID_PUB_KEY)
        .header("address", TEST_VALID_ADDRESS)
        .header("signature", TEST_VALID_SIG)
        .path("/get_data");

    let db_stub = Arc::new(Mutex::new(DbStub::init("").await.unwrap()));
    let cache_stub = Arc::new(Mutex::new(DbStub::init("").await.unwrap()));
    let cfilter = Arc::new(Mutex::new(cuckoofilter::CuckooFilter::new()));

    db_stub
        .lock()
        .await
        .set_data(TEST_VALID_ADDRESS, "{\"Hello\":20}")
        .await
        .unwrap();
    cache_stub
        .lock()
        .await
        .set_data(TEST_VALID_ADDRESS, "{\"Hello\":20}")
        .await
        .unwrap();
    cfilter.lock().await.add(TEST_VALID_ADDRESS).unwrap();

    //
    // Act
    //
    let filter = routes::get_data(db_stub, cache_stub, cfilter).recover(handle_rejection);
    let res = request.reply(&filter).await;

    //
    // Assert
    //
    assert_eq!(res.status(), 200);
    assert_eq!(
        res.body(),
        "{\"status\":\"Success\",\"reason\":\"Data retrieved successfully\",\"route\":\"get_data\",\"content\":\"\"}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_set_data() {
    //
    // Arrange
    //
    let req_body = "{\"address\":\"0x123\",\"data\":\"{\\\"Hello\\\":20}\"}";

    let request = warp::test::request()
        .method("POST")
        .header("public_key", TEST_VALID_PUB_KEY)
        .header("address", TEST_VALID_ADDRESS)
        .header("signature", TEST_VALID_SIG)
        .body(req_body)
        .path("/set_data");

    let db_stub = Arc::new(Mutex::new(DbStub::init("").await.unwrap()));
    let cache_stub = Arc::new(Mutex::new(DbStub::init("").await.unwrap()));
    let cfilter = Arc::new(Mutex::new(cuckoofilter::CuckooFilter::new()));

    //
    // Act
    //
    let filter =
        routes::set_data(db_stub, cache_stub, cfilter, 1000, 600).recover(handle_rejection);
    let res = request.reply(&filter).await;

    //
    // Assert
    //
    assert_eq!(res.status(), 200);
    assert_eq!(
        res.body(),
        "{\"status\":\"Success\",\"reason\":\"Data set successfully\",\"route\":\"set_data\",\"content\":\"0x123\"}"
    );
}
