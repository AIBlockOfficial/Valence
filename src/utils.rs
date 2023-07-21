use crate::crypto::sign_ed25519 as sign;
use crate::crypto::sign_ed25519::{PublicKey, Signature};

// Function to validate the signature using Ed25519
pub fn validate_signature(public_key: &str, msg: &str, signature: &str) -> bool {
    let pk = PublicKey::from_slice(public_key.as_bytes()).unwrap();
    let signature = Signature::from_slice(signature.as_bytes()).unwrap();

    sign::verify_detached(&signature, msg.as_bytes(), &pk)
}