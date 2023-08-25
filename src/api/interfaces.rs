use std::sync::Arc;
use futures::lock::Mutex;
use std::collections::hash_map::DefaultHasher;
use crate::db::redis_cache::RedisCacheConn;
use crate::db::mongo_db::MongoDbConn;

/// ========= TYPE ABSTRACTIONS ========= ///

pub type CacheConnection = Arc<Mutex<RedisCacheConn>>;
pub type DbConnection = Arc<Mutex<MongoDbConn>>;
pub type CFilterConnection = Arc<Mutex<cuckoofilter::CuckooFilter<DefaultHasher>>>;