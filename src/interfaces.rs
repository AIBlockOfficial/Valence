use serde::{Deserialize, Serialize};

// Define a struct to hold the data (public key, address, signature)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRequestData {
    pub public_key: String,
    pub address: String,
    pub signature: String,
}


// Custom error type for invalid signature
#[derive(Debug)]
pub struct InvalidSignature;

#[derive(Debug)]
pub struct DBInsertionFailed;
