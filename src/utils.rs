use crate::constants::{
    CONFIG_FILE, DRUID_CHARSET, DRUID_LENGTH, SETTINGS_BODY_LIMIT, SETTINGS_CACHE_PASSWORD,
    SETTINGS_CACHE_PORT, SETTINGS_CACHE_URL, SETTINGS_DB_PASSWORD, SETTINGS_DB_PORT,
    SETTINGS_DB_URL, SETTINGS_DEBUG, SETTINGS_EXTERN_PORT,
};
use crate::interfaces::EnvConfig;
use chrono::prelude::*;
use rand::Rng;

/// Loads the config file
pub fn load_config() -> EnvConfig {
    let settings = config::Config::builder().add_source(config::File::with_name(CONFIG_FILE));

    match settings.build() {
        Ok(config) => EnvConfig {
            debug: config.get_bool("debug").unwrap_or(SETTINGS_DEBUG),
            extern_port: config
                .get_int("extern_port")
                .unwrap_or(SETTINGS_EXTERN_PORT as i64) as u16,
            db_url: config
                .get_string("db_url")
                .unwrap_or(SETTINGS_DB_URL.to_string()),
            db_port: config
                .get_string("db_port")
                .unwrap_or(SETTINGS_DB_PORT.to_string()),
            db_password: config
                .get_string("db_password")
                .unwrap_or(SETTINGS_DB_PASSWORD.to_string()),
            cache_url: config
                .get_string("cache_url")
                .unwrap_or(SETTINGS_CACHE_URL.to_string()),
            cache_port: config
                .get_string("cache_port")
                .unwrap_or(SETTINGS_CACHE_PORT.to_string()),
            cache_password: config
                .get_string("cache_password")
                .unwrap_or(SETTINGS_CACHE_PASSWORD.to_string()),
            body_limit: config
                .get_int("body_limit")
                .unwrap_or(SETTINGS_BODY_LIMIT as i64) as u64,
        },
        Err(e) => {
            panic!("Failed to load config file with error: {e}")
        }
    }
}

/// Constructs a 16 byte DRUID string
pub fn construct_druid() -> String {
    let mut rng = rand::thread_rng();
    let random_string: String = (0..DRUID_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..DRUID_CHARSET.len());
            DRUID_CHARSET[idx] as char
        })
        .collect();

    random_string
}

/// Constructs a string-formatted date
pub fn construct_formatted_date() -> String {
    let utc_now: DateTime<Utc> = Utc::now();
    utc_now.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn print_welcome(db_addr: &str, cache_addr: &str) {
    println!("");
    println!(
        " 
    
  __     __     ______     ______     __   __   ______     ______    
 /\\ \\  _ \\ \\   /\\  ___\\   /\\  __ \\   /\\ \\ / /  /\\  ___\\   /\\  == \\   
 \\ \\ \\/ |.\\ \\  \\ \\  __\\   \\ \\  __ \\  \\ \\ \\'/   \\ \\  __\\   \\ \\  __<   
  \\ \\__/|.~\\_\\  \\ \\_____\\  \\ \\_\\ \\_\\  \\ \\__|    \\ \\_____\\  \\ \\_\\ \\_\\ 
   \\/_/   \\/_/   \\/_____/   \\/_/\\/_/   \\/_/      \\/_____/   \\/_/ /_/ 
                                                                     
 "
    );

    println!("");
    println!("Connecting to cache at {}", cache_addr);
    println!("Connecting to DB at {}", db_addr);
    println!("");
}
