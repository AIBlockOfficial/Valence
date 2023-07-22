use serde::{Serialize, Deserialize};
use crate::crypto::sign_ed25519 as sign;
use crate::crypto::sign_ed25519::{PublicKey, Signature};

// Function to validate the signature using Ed25519
pub fn validate_signature(public_key: &str, msg: &str, signature: &str) -> bool {
    let pk = PublicKey::from_slice(public_key.as_bytes()).unwrap();
    let signature = Signature::from_slice(signature.as_bytes()).unwrap();

    sign::verify_detached(&signature, msg.as_bytes(), &pk)
}

// Function to serialize data
pub fn serialize_data<T: Serialize + for<'a> Deserialize<'a>>(data: T) -> String {
    serde_json::to_string(&data).unwrap()
}

// Function to deserialize data
pub fn deserialize_data<T: Serialize + for<'a> Deserialize<'a>>(data: String) -> T {
    serde_json::from_str(&data).unwrap()
}