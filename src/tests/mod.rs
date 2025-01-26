pub mod constants;
pub mod interfaces;

use crate::interfaces::SetRequestData;
use crate::api::routes;
use crate::db::handler::KvStoreConnection;
use crate::tests::constants::{TEST_VALID_ADDRESS, TEST_VALID_PUB_KEY, TEST_VALID_SIG};
use crate::tests::interfaces::DbStub;
use futures::lock::Mutex;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use valence_core::api::utils::handle_rejection;
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

    let test_value = "{\"Hello\":20}".to_string();

    db_stub
        .lock()
        .await
        .set_data(TEST_VALID_ADDRESS, "blah", test_value.clone())
        .await
        .unwrap();
    cache_stub
        .lock()
        .await
        .set_data(TEST_VALID_ADDRESS, "blah", test_value)
        .await
        .unwrap();
    cfilter.lock().await.add(TEST_VALID_ADDRESS).unwrap();

    //
    // Act
    //
    let filter = routes::get_data(db_stub, cache_stub, cfilter).recover(handle_rejection);
    let res = request.reply(&filter).await;

    println!("{:?}", res.body());

    //
    // Assert
    //
    assert_eq!(res.status(), 200);
    assert_eq!(
        res.body(),
        "{\"status\":\"Success\",\"reason\":\"Data retrieved successfully\",\"route\":\"get_data\",\"content\":{}}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_set_data() {
    //
    // Arrange
    //
    let req_body = "{\"address\":\"0x123\",\"data\":\"{\\\"Hello\\\":20}\", \"data_id\":\"id\"}";

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

#[tokio::test(flavor = "current_thread")]
async fn test_set_data_multiple_requests() {
    let db = Arc::new(Mutex::new(DbStub::init("").await.unwrap()));
    let cache = Arc::new(Mutex::new(DbStub::init("").await.unwrap()));
    let cfilter = Arc::new(Mutex::new(cuckoofilter::CuckooFilter::new()));
    let mut expected_entries = std::collections::HashMap::new();

    for _ in 0..3 {
        let key = Uuid::new_v4().to_string();
        let value = Uuid::new_v4().to_string();
        let data_id = Uuid::new_v4().to_string();

        let set_request = SetRequestData {
            address: TEST_VALID_ADDRESS.to_string(),
            data: json!({ &key: &value }),
            data_id: data_id.clone(),
        };

        let request = warp::test::request()
            .method("POST")
            .header("public_key", TEST_VALID_PUB_KEY)
            .header("address", TEST_VALID_ADDRESS)
            .header("signature", TEST_VALID_SIG)
            .path("/set_data")
            .json(&set_request);

        let filter = routes::set_data(db.clone(), cache.clone(), cfilter.clone(), 1000, 600)
            .recover(handle_rejection);
        let res = request.reply(&filter).await;

        assert_eq!(res.status(), 200);
        let res_body: serde_json::Value = serde_json::from_str(std::str::from_utf8(res.body()).unwrap()).unwrap();
        assert_eq!(res_body["status"], "Success");
        assert_eq!(res_body["reason"], "Data set successfully");
        assert_eq!(res_body["route"], "set_data");
        assert_eq!(res_body["content"], TEST_VALID_ADDRESS);

        expected_entries.insert(data_id.clone(), (key, value));

        let mut db_lock = db.lock().await;
        for (id, (expected_key, expected_value)) in &expected_entries {
            let stored_data = db_lock.get_data::<serde_json::Value>(TEST_VALID_ADDRESS, Some(id)).await.unwrap().unwrap();
            assert_eq!(stored_data[expected_key], *expected_value);
        }
    }
}