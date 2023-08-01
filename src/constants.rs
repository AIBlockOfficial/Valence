pub const CONFIG_FILE: &str = ".env";
pub const SETTINGS_DEBUG: bool = false;
pub const SETTINGS_EXTERN_PORT: &str = "3030";
pub const SETTINGS_DB_URL: &str = "redis://127.0.0.1";
pub const SETTINGS_DB_PORT: &str = "6379";
pub const SETTINGS_DB_PASSWORD: &str = "password";

/// ==== DRUID CONSTANTS ==== ///

pub const DRUID_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

pub const DRUID_LENGTH: usize = 16;