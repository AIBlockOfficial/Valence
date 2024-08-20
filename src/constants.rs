/// ==== CONFIG ==== ///

pub const CONFIG_FILE: &str = "config.toml";
pub const SETTINGS_DEBUG: bool = false;
pub const SETTINGS_EXTERN_PORT: u16 = 8080;
pub const SETTINGS_DB_PROTOCOL: &str = "mongodb";
pub const SETTINGS_DB_URL: &str = "127.0.0.1";
pub const SETTINGS_DB_PORT: &str = "12701";
pub const SETTINGS_DB_USER: &str = "root";
pub const SETTINGS_DB_PASSWORD: &str = "example";
pub const SETTINGS_CACHE_URL: &str = "redis://127.0.0.1";
pub const SETTINGS_CACHE_PORT: &str = "6379";
pub const SETTINGS_CACHE_PASSWORD: &str = "password";
pub const SETTINGS_BODY_LIMIT: u64 = 4096;
pub const SETTINGS_CACHE_TTL: u64 = 600;

/// ==== DRUID ==== ///

pub const DRUID_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

pub const DRUID_LENGTH: usize = 16;

/// ==== STORAGE ==== ///

pub const DB_KEY: &str = "default";
pub const CUCKOO_FILTER_KEY: &str = "cuckoo_filter";
pub const CUCKOO_FILTER_VALUE_ID: &str = "cuckoo_filter_id";
