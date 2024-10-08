use std::collections::HashMap;

use crate::db::handler::{CacheHandler, KvStoreConnection};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use valence_core::utils::serialize_data;

//========== STUB INTERFACES ==========//

#[derive(Serialize, Deserialize)]
pub struct GetReturn {
    pub status: String,
    pub reason: String,
    pub route: String,
    pub content: String,
}

/// A stub for the database connection
#[derive(Clone)]
pub struct DbStub {
    data: Option<String>,
}

#[async_trait]
impl CacheHandler for DbStub {
    async fn expire_entry(
        &mut self,
        _key: &str,
        _seconds: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[async_trait]
impl KvStoreConnection for DbStub {
    async fn init(_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(DbStub { data: None })
    }

    async fn get_data<T: DeserializeOwned>(
        &mut self,
        _key: &str,
        _value_id: Option<&str>,
    ) -> Result<Option<HashMap<String, T>>, Box<dyn std::error::Error + Send + Sync>> {
        let data = match self.data.clone() {
            Some(d) => d,
            None => {
                println!("No data found");
                return Ok(None);
            }
        };

        Ok(Some(get_de_data(data)))
    }

    async fn del_data(
        &mut self,
        _key: &str,
        _value_id: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.data = None;

        Ok(())
    }

    async fn set_data_with_expiry<T: Serialize + Send>(
        &mut self,
        _key: &str,
        _value_id: &str,
        value: T,
        _seconds: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.data = Some(serialize_data(&value));

        Ok(())
    }

    async fn set_data<T: Serialize + Send>(
        &mut self,
        _key: &str,
        _value_id: &str,
        value: T,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.data = Some(serialize_data(&value));

        Ok(())
    }
}

fn get_de_data<T: DeserializeOwned>(v: String) -> HashMap<String, T> {
    let value: serde_json::Value = match serde_json::from_str(&v) {
        Ok(v) => v,
        Err(_) => {
            println!("Failed to deserialize data");
            return HashMap::new();
        }
    };
    let de_map: HashMap<String, T> =
        serde_json::from_str(&value.as_str().unwrap()).unwrap_or(HashMap::new());

    de_map
}
