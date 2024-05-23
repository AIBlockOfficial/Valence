use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use valence_core::db::handler::{CacheHandler, KvStoreConnection};
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
    ) -> Result<Option<Vec<T>>, Box<dyn std::error::Error + Send + Sync>> {
        if self.data.is_none() {
            return Ok(None);
        }

        let data = match self.data.clone() {
            Some(d) => d,
            None => return Ok(None),
        };

        match get_de_data::<Vec<T>>(data) {
            Ok(d) => Ok(Some(d)),
            Err(_) => Ok(None),
        }
    }

    async fn delete_data(
        &mut self,
        _key: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.data = None;

        Ok(())
    }

    async fn set_data_with_expiry<T: Serialize + Send>(
        &mut self,
        _key: &str,
        value: T,
        _seconds: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.data = Some(serialize_data(&value));

        Ok(())
    }

    async fn set_data<T: Serialize + Send>(
        &mut self,
        _key: &str,
        value: T,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.data = Some(serialize_data(&value));

        Ok(())
    }
}

fn get_de_data<T: DeserializeOwned>(v: String) -> Result<T, serde_json::Error> {
    serde_json::from_str(&v)
}
