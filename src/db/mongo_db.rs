use futures::lock::Mutex;
use mongodb::{options::ClientOptions, Client, results::InsertOneResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MongoDbIndex {
    pub db_name: String,
    pub coll_name: String,
}

/// Initialises a MongoDB connection
///
/// ### Arguments
///
/// * `url` - URL of the MongoDB instance to connect to
pub async fn init_db_conn(url: &str) -> Arc<Mutex<Client>> {
    let client_options = match ClientOptions::parse(url).await {
        Ok(client_options) => client_options,
        Err(e) => panic!("Failed to connect to MongoDB instance with error: {}", e),
    };

    let client = match Client::with_options(client_options) {
        Ok(client) => client,
        Err(e) => panic!("Failed to connect to MongoDB instance with error: {}", e),
    };

    Arc::new(Mutex::new(client))
}

/// Creates a collection within a database
///
/// ### Arguments
///
/// * `client` - MongoDB client
/// * `db_name` - Name of the database to create the collection in
/// * `coll_name` - Name of the collection to create
pub async fn create_collection(client: &Client, db_name: &str, coll_name: &str) {
    let db = client.database(db_name);
    match db.create_collection(coll_name, None).await {
        Ok(_) => (),
        Err(e) => panic!("Failed to create MongoDB collection with error: {}", e),
    }
}

/// Inserts a document into a collection
///
/// ### Arguments
///
/// * `client` - MongoDB client
/// * `db_name` - Name of the database to create the collection in
/// * `coll_name` - Name of the collection to create
/// * `doc` - Document to insert into the collection
pub async fn insert_document<'a, T: Serialize + Deserialize<'a>>(
    client: &Arc<Mutex<Client>>,
    db_name: &str,
    coll_name: &str,
    doc: T,
) -> Result<InsertOneResult, mongodb::error::Error> {
    let client_ref = client.lock().await;
    let db = client_ref.database(db_name);
    let coll = db.collection(coll_name);

    coll.insert_one(doc, None).await
}
