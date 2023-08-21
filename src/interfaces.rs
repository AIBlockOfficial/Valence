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

pub struct AddOrderRequestData {}

pub struct EnvConfig {
    pub debug: bool,
    pub extern_port: String,
    pub db_url: String,
    pub db_port: String,
    pub db_password: String,
    pub cache_url: String,
    pub cache_port: String,
    pub cache_password: String,
}

// Custom error type for invalid signature
#[derive(Debug)]
pub struct InvalidSignature;

#[derive(Debug)]
pub struct DBInsertionFailed;

#[derive(Debug)]
pub struct CacheInsertionFailed;

#[derive(Debug)]
pub struct CuckooFilterInsertionFailed;

#[derive(Debug)]
pub struct CuckooFilterLookupFailed;
