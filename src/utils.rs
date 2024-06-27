use crate::constants::{
    CONFIG_FILE, CUCKOO_FILTER_KEY, CUCKOO_FILTER_VALUE_ID, DRUID_CHARSET, DRUID_LENGTH,
    SETTINGS_BODY_LIMIT, SETTINGS_CACHE_PASSWORD, SETTINGS_CACHE_PORT, SETTINGS_CACHE_TTL,
    SETTINGS_CACHE_URL, SETTINGS_DB_PASSWORD, SETTINGS_DB_PORT, SETTINGS_DB_PROTOCOL,
    SETTINGS_DB_URL, SETTINGS_DEBUG, SETTINGS_EXTERN_PORT,
};
use crate::interfaces::EnvConfig;
use crate::db::handler::KvStoreConnection;
use crate::db::mongo_db::MongoDbConn;
use crate::db::redis_cache::RedisCacheConn;
use chrono::prelude::*;
use cuckoofilter::{CuckooFilter, ExportedCuckooFilter};
use futures::lock::Mutex;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::sync::Arc;
use tracing::info;

// ========== STORAGE SERIALIZATION FOR CUCKOO FILTER ========== //

/// Serializable struct for cuckoo filter
#[derive(Serialize, Deserialize, Debug, Clone)]
struct StorageReadyCuckooFilter {
    values: Vec<u8>,
    length: usize,
}

impl From<ExportedCuckooFilter> for StorageReadyCuckooFilter {
    fn from(cf: ExportedCuckooFilter) -> Self {
        StorageReadyCuckooFilter {
            values: cf.values,
            length: cf.length,
        }
    }
}

impl Into<ExportedCuckooFilter> for StorageReadyCuckooFilter {
    fn into(self) -> ExportedCuckooFilter {
        ExportedCuckooFilter {
            values: self.values,
            length: self.length,
        }
    }
}

// ========== DB UTILS ========== //

/// Constructs a MongoDB connection
///
/// ### Arguments
///
/// * `url` - The URL to connect to
pub async fn construct_mongodb_conn(url: &str) -> Arc<Mutex<MongoDbConn>> {
    let mongo_conn = match MongoDbConn::init(url).await {
        Ok(conn) => conn,
        Err(e) => panic!("Failed to connect to MongoDB with error: {}", e),
    };

    Arc::new(Mutex::new(mongo_conn))
}

/// Constructs a Redis cache connection
///
/// ### Arguments
///
/// * `url` - The URL to connect to
pub async fn construct_redis_conn(url: &str) -> Arc<Mutex<RedisCacheConn>> {
    let redis_conn = match RedisCacheConn::init(url).await {
        Ok(conn) => conn,
        Err(e) => panic!("Failed to connect to MongoDB with error: {}", e),
    };

    Arc::new(Mutex::new(redis_conn))
}

// ========== CUCKOO FILTER UTILS ========== //

/// Saves the cuckoo filter to disk
///
/// ### Arguments
///
/// * `cf` - The cuckoo filter to save
/// * `db` - The database connection
pub async fn save_cuckoo_filter_to_disk<T: KvStoreConnection>(
    cf: &CuckooFilter<DefaultHasher>,
    db: Arc<Mutex<T>>,
) -> Result<(), String> {
    let cuckoo_export = cf.export();
    let serializable_cuckoo: StorageReadyCuckooFilter = cuckoo_export.into();
    let mut db_lock = db.lock().await;

    match db_lock
        .set_data(
            CUCKOO_FILTER_KEY,
            CUCKOO_FILTER_VALUE_ID,
            serializable_cuckoo,
        )
        .await
    {
        Ok(_) => {
            info!("Cuckoo filter saved to disk successfully");
            Ok(())
        }
        Err(e) => Err(format!(
            "Failed to save cuckoo filter to disk with error: {}",
            e
        )),
    }
}

/// Loads the cuckoo filter from disk
///
/// ### Arguments
///
/// * `db` - The database connection
pub async fn load_cuckoo_filter_from_disk<T: KvStoreConnection>(
    db: Arc<Mutex<T>>,
) -> Result<CuckooFilter<DefaultHasher>, String> {
    let mut db_lock = db.lock().await;

    match db_lock
        .get_data::<StorageReadyCuckooFilter>(CUCKOO_FILTER_KEY, Some(CUCKOO_FILTER_VALUE_ID))
        .await
    {
        Ok(data) => match data {
            Some(data) => {
                let cf: StorageReadyCuckooFilter =
                    data.get(CUCKOO_FILTER_VALUE_ID).unwrap().clone();
                info!("Found existing cuckoo filter. Loaded from disk successfully");

                let cfe: ExportedCuckooFilter = cf.into();
                let cf: CuckooFilter<DefaultHasher> = CuckooFilter::from(cfe);

                Ok(cf)
            }
            None => Err("No cuckoo filter found in DB".to_string()),
        },
        Err(e) => Err(format!(
            "Failed to load cuckoo filter from disk with error: {}",
            e
        )),
    }
}

/// Initializes the cuckoo filter
///
/// ### Arguments
///
/// * `db` - The database connection
pub async fn init_cuckoo_filter<T: KvStoreConnection>(
    db: Arc<Mutex<T>>,
) -> Result<CuckooFilter<DefaultHasher>, String> {
    match load_cuckoo_filter_from_disk(db.clone()).await {
        Ok(cf) => Ok(cf),
        Err(_) => {
            info!("No cuckoo filter found in DB, initializing new one");
            let cf = CuckooFilter::new();
            save_cuckoo_filter_to_disk(&cf, db).await.unwrap();

            info!("New cuckoo filter saved to database");

            Ok(cf)
        }
    }
}

// ========== CONFIG UTILS ========== //

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
            db_user: config
                .get_string("db_user")
                .unwrap_or(SETTINGS_DB_URL.to_string()),
            db_protocol: config
                .get_string("db_protocol")
                .unwrap_or(SETTINGS_DB_PROTOCOL.to_string()),
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
            cache_ttl: config
                .get_int("cache_ttl")
                .unwrap_or(SETTINGS_CACHE_TTL as i64) as usize,
            market: config.get_bool("market").unwrap_or(false),
        },
        Err(e) => {
            panic!("Failed to load config file with error: {e}")
        }
    }
}

// ========== MISC UTILS ========== //

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
    println!();
    println!(
        " 
                     ___                         ___           ___           ___           ___     
        ___         /  /\\                       /  /\\         /__/\\         /  /\\         /  /\\    
       /__/\\       /  /::\\                     /  /:/_        \\  \\:\\       /  /:/        /  /:/_   
       \\  \\:\\     /  /:/\\:\\    ___     ___    /  /:/ /\\        \\  \\:\\     /  /:/        /  /:/ /\\  
        \\  \\:\\   /  /:/~/::\\  /__/\\   /  /\\  /  /:/ /:/_   _____\\__\\:\\   /  /:/  ___   /  /:/ /:/_ 
    ___  \\__\\:\\ /__/:/ /:/\\:\\ \\  \\:\\ /  /:/ /__/:/ /:/ /\\ /__/::::::::\\ /__/:/  /  /\\ /__/:/ /:/ /\\
   /__/\\ |  |:| \\  \\:\\/:/__\\/  \\  \\:\\  /:/  \\  \\:\\/:/ /:/ \\  \\:\\~~\\~~\\/ \\  \\:\\ /  /:/ \\  \\:\\/:/ /:/
   \\  \\:\\|  |:|  \\  \\::/        \\  \\:\\/:/    \\  \\::/ /:/   \\  \\:\\  ~~~   \\  \\:\\  /:/   \\  \\::/ /:/ 
    \\  \\:\\__|:|   \\  \\:\\         \\  \\::/      \\  \\:\\/:/     \\  \\:\\        \\  \\:\\/:/     \\  \\:\\/:/  
     \\__\\::::/     \\  \\:\\         \\__\\/        \\  \\::/       \\  \\:\\        \\  \\::/       \\  \\::/   
         ~~~~       \\__\\/                       \\__\\/         \\__\\/         \\__\\/         \\__\\/    
                                                                     
 "
    );

    println!();
    println!("Connecting to cache at {}", cache_addr);
    println!("Connecting to DB at {}", db_addr);
    println!();
}
