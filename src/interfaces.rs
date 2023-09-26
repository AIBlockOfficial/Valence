use serde::{Deserialize, Serialize};

// Define a struct to hold the data (public key, address, signature)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRequestData {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetRequestData {
    pub address: String,
    pub data: String,
}

pub struct EnvConfig {
    pub debug: bool,
    pub extern_port: u16,
    pub db_url: String,
    pub db_port: String,
    pub db_password: String,
    pub cache_url: String,
    pub cache_port: String,
    pub cache_password: String,
    pub body_limit: u64,

    pub market: bool,
}
