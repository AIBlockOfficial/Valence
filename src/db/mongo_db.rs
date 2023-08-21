use futures::lock::Mutex;
use mongodb::{ options::ClientOptions, Client, results::InsertOneResult };
use serde::{ Deserialize, Serialize };
use std::sync::Arc;
use async_trait::async_trait;

use super::handler::DbConnection;

#[derive(Debug, Clone)]
pub struct MongoDbIndex {
    pub db_name: String,
    pub coll_name: String,
}

#[derive(Debug, Clone)]
pub struct MongoDbConn {
    pub client: Arc<Mutex<Client>>,
}

#[async_trait]
impl DbConnection for MongoDbConn {
    type ConnectionResult = MongoDbConn;
    type InsertCollResult = ();
    type InsertDocResult = ();

    async fn init(&mut self, url: &str) -> Self::ConnectionResult {
        let client_options = match ClientOptions::parse(url).await {
            Ok(client_options) => client_options,
            Err(e) => panic!("Failed to connect to MongoDB instance with error: {}", e),
        };

        let client = match Client::with_options(client_options) {
            Ok(client) => client,
            Err(e) => panic!("Failed to connect to MongoDB instance with error: {}", e),
        };

        MongoDbConn {
            client: Arc::new(Mutex::new(client)),
        }
    }

    async fn insert_collection(
        &mut self,
        db_name: &str,
        coll_name: &str
    ) -> Self::InsertCollResult {
        let client = self.client.lock().await;
        let db = client.database(db_name);
        match db.create_collection(coll_name, None).await {
            Ok(_) => (),
            Err(e) => panic!("Failed to create MongoDB collection with error: {}", e),
        }
    }

    async fn insert_document<'a, T: Serialize + Deserialize<'a> + std::marker::Send>(
        &mut self,
        db_name: &str,
        coll_name: &str,
        doc: T
    ) -> Self::InsertDocResult {
        let client = self.client.lock().await;
        let coll = client.database(db_name).collection(coll_name);

        let insertion = match coll.insert_one(doc, None).await {
            Ok(_) => (),
            Err(e) => panic!("Failed to insert document into MongoDB collection with error: {}", e),
        };

        insertion
    }
}
