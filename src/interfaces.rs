use serde::{Deserialize, Serialize};
use serde_json::Value;

// Define a struct to hold the data (public key, address, signature)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRequestData {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetRequestData {
    pub address: String,
    pub data: Value,
    pub data_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSaveData {
    pub address: String,
    pub data: Value,
}

pub struct EnvConfig {
    pub debug: bool,
    pub extern_port: u16,
    pub db_protocol: String,
    pub db_user: String,
    pub db_url: String,
    pub db_port: String,
    pub db_password: String,
    pub cache_url: String,
    pub cache_port: String,
    pub cache_password: String,
    pub body_limit: u64,
    pub cache_ttl: usize,

    pub market: bool,
}
