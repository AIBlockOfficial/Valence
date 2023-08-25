/// ==== CONFIG ==== ///

pub const CONFIG_FILE: &str = ".env";
pub const SETTINGS_DEBUG: bool = false;
pub const SETTINGS_EXTERN_PORT: u16 = 8080;
pub const SETTINGS_DB_URL: &str = "mongodb://127.0.0.1";
pub const SETTINGS_DB_PORT: &str = "12701";
pub const SETTINGS_DB_PASSWORD: &str = "password";
pub const SETTINGS_CACHE_URL: &str = "redis://127.0.0.1";
pub const SETTINGS_CACHE_PORT: &str = "6379";
pub const SETTINGS_CACHE_PASSWORD: &str = "password";

/// ==== DRUID ==== ///

pub const DRUID_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

pub const DRUID_LENGTH: usize = 16;

/// ==== STORAGE ==== ///

pub const DB_KEY: &str = "default";
