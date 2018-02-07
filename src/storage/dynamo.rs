use std::collections::HashMap;
use std::time::{Duration, Instant};

use error::BannerError;
use hash_cache::HashCache;
use store::Store;

#[derive(Debug)]
pub struct DynamoStore<T> {
    data: HashCache<T>,
}

impl<T> DynamoStore<T> {
    pub fn new() -> DynamoStore<T> {
        DynamoStore {
            data: HashCache::new(Duration::new(0, 0)),
        }
    }
}

impl<T> From<HashMap<String, (T, Instant)>> for DynamoStore<T> {
    fn from(map: HashMap<String, (T, Instant)>) -> DynamoStore<T> {
        DynamoStore { data: map.into() }
    }
}

impl<T: Clone> Store for DynamoStore<T> {
    type Item = T;
    type Error = BannerError;

    fn get(&self, key: &str) -> Result<Option<T>, BannerError> {
        self.data.get(key)
    }

    fn get_all(&self) -> Result<HashMap<String, T>, BannerError> {
        self.data.get_all()
    }

    fn delete(&self, key: &str) -> Result<Option<T>, BannerError> {
        self.data.remove(key)
    }

    fn upsert(&self, key: &str, item: &T) -> Result<Option<T>, BannerError> {
        self.data.insert(key, item)
    }
}
