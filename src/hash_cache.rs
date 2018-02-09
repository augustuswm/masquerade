use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant};

use error::BannerError;

#[derive(Clone, Debug)]
pub struct HashCache<T> {
    cache: Arc<RwLock<HashMap<String, (T, Instant)>>>,
    duration: Duration,
}

impl<T> From<HashMap<String, (T, Instant)>> for HashCache<T> {
    fn from(map: HashMap<String, (T, Instant)>) -> HashCache<T> {
        HashCache {
            cache: Arc::new(RwLock::new(map)),
            duration: Duration::new(0, 0),
        }
    }
}

pub type CacheResult<T> = Result<T, BannerError>;

impl<T> HashCache<T> {
    pub fn new(duration: Duration) -> HashCache<T> {
        HashCache {
            cache: Arc::new(RwLock::new(HashMap::new())),
            duration: duration,
        }
    }

    pub fn reader(&self) -> CacheResult<RwLockReadGuard<HashMap<String, (T, Instant)>>> {
        self.cache.read().map_err(|_| {
            error!("Failed to acquire read guard for cache failed due to poisoning");
            BannerError::CachePoisonedError
        })
    }

    pub fn writer(&self) -> CacheResult<RwLockWriteGuard<HashMap<String, (T, Instant)>>> {
        self.cache.write().map_err(|_| {
            error!("Failed to acquire write guard for cache failed due to poisoning");
            BannerError::CachePoisonedError
        })
    }

    fn ignore_dur(&self) -> bool {
        self.duration.as_secs() as f64 + self.duration.subsec_nanos() as f64 == 0.0
    }
}

impl<T: Clone> HashCache<T> {
    pub fn get<'a, S: Into<&'a str>>(&self, key: S) -> CacheResult<Option<T>> {
        self.reader().map(|reader| {
            let entry = reader.get(key.into());

            match entry {
                Some(&(ref val, created)) => {
                    if self.ignore_dur() || created.elapsed() <= self.duration {
                        Some(val.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
    }

    pub fn get_all(&self) -> CacheResult<HashMap<String, T>> {
        let mut res: HashMap<String, T> = HashMap::new();

        self.reader().map(|reader| {
            for (k, &(ref f, created)) in reader.iter() {
                if self.ignore_dur() || created.elapsed() <= self.duration {
                    res.insert(k.clone(), f.clone());
                }
            }

            res
        })
    }

    pub fn insert<S: Into<String>>(&self, key: S, val: &T) -> CacheResult<Option<T>> {
        self.writer().map(|mut writer| {
            writer
                .insert(key.into(), (val.clone(), Instant::now()))
                .map(|(v, _)| v)
        })
    }

    pub fn remove<'a, S: Into<&'a str>>(&self, key: S) -> CacheResult<Option<T>> {
        self.writer()
            .map(|mut writer| writer.remove(key.into()).map(|(v, _)| v))
    }

    pub fn clear(&self) -> CacheResult<()> {
        self.writer().map(|mut writer| writer.clear())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_then_read() {
        let cache: HashCache<Vec<u8>> = HashCache::new(Duration::new(5, 0));
        let val = vec![1, 2, 3];
        cache.insert("3", &val);
        assert_eq!(Ok(Some(val)), cache.get("3"));
    }
}
