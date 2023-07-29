use crate::crypto::sign_ed25519 as sign;
use crate::crypto::sign_ed25519::{ PublicKey, Signature };
use crate::interfaces::EnvConfig;
use crate::constants::{
    CONFIG_FILE,
    SETTINGS_DEBUG,
    SETTINGS_EXTERN_PORT,
    SETTINGS_DB_PORT,
    SETTINGS_DB_PASSWORD,
    SETTINGS_DB_URL,
};
use serde::{ Deserialize, Serialize };

/// Loads the config file
pub fn load_config() -> EnvConfig {
    let settings = config::Config::builder().add_source(config::File::with_name(CONFIG_FILE));

    match settings.build() {
        Ok(config) => {
            EnvConfig {
                debug: config.get_bool("debug").unwrap_or(SETTINGS_DEBUG),
                extern_port: config
                    .get_string("extern_port")
                    .unwrap_or(SETTINGS_EXTERN_PORT.to_string()),
                db_url: config.get_string("db_url").unwrap_or(SETTINGS_DB_URL.to_string()),
                db_port: config.get_string("db_port").unwrap_or(SETTINGS_DB_PORT.to_string()),
                db_password: config
                    .get_string("db_password")
                    .unwrap_or(SETTINGS_DB_PASSWORD.to_string()),
            }
        }
        Err(e) => { panic!("Failed to load config file with error: {}", e) }
    }
}

/// Function to validate the signature using Ed25519
pub fn validate_signature(public_key: &str, msg: &str, signature: &str) -> bool {
    let pk = PublicKey::from_slice(public_key.as_bytes()).unwrap();
    let signature = Signature::from_slice(signature.as_bytes()).unwrap();

    sign::verify_detached(&signature, msg.as_bytes(), &pk)
}

/// Function to serialize data
pub fn serialize_data<T: Serialize + for<'a> Deserialize<'a>>(data: T) -> String {
    serde_json::to_string(&data).unwrap()
}

/// Function to deserialize data
pub fn deserialize_data<T: Serialize + for<'a> Deserialize<'a>>(data: String) -> T {
    serde_json::from_str(&data).unwrap()
}
