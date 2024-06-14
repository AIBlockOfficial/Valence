use futures::lock::Mutex;
use serde_json::Value;
use std::sync::Arc;
use tracing::info;
use valence_core::api::errors::ApiErrorType;
use valence_core::api::responses::{json_serialize_embed, CallResponse, JsonReply};
use valence_core::db::handler::KvStoreConnection;

/// Retrieve data from the database
///
/// ### Arguments
///
/// * `db` - Database connection
/// * `address` - Address to retrieve data from
pub async fn retrieve_from_db<D: KvStoreConnection + Clone + Send + 'static>(
    db: Arc<Mutex<D>>,
    address: &str,
) -> Result<JsonReply, JsonReply> {
    let r = CallResponse::new("get_data");
    info!("RETRIEVE_FROM_DB requested with address: {:?}", address);

    let db_result: Result<Option<Vec<Value>>, _> = db.lock().await.get_data(&address).await;

    match db_result {
        Ok(data) => {

            let elems = data
                .into_iter()
                .map(serde_json::from_str)
                .collect::<Result<Vec<Value>>>()
                .unwrap();

            let json_val = Value::Array(elems);

            return r.into_ok("Data retrieved successfully", json_serialize_embed(data))
        },
        Err(_) => r.into_err_internal(ApiErrorType::Generic(
            "Full Valence chain retrieval failed".to_string(),
        )),
    }
}
