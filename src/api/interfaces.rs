use beacon_core::db::mongo_db::MongoDbConn;
use beacon_core::db::redis_cache::RedisCacheConn;
use futures::lock::Mutex;
use std::collections::hash_map::DefaultHasher;
use std::sync::Arc;

/// ========= TYPE ABSTRACTIONS ========= ///

pub type CacheConnection = Arc<Mutex<RedisCacheConn>>;
pub type DbConnection = Arc<Mutex<MongoDbConn>>;
pub type CFilterConnection = Arc<Mutex<cuckoofilter::CuckooFilter<DefaultHasher>>>;
