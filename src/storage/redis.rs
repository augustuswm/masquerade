use redis::{cmd, Client, Commands, Connection, ErrorKind, FromRedisValue, RedisResult,
            ToRedisArgs, Value as RedisValue};
use serde_json;

use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

use error::BannerError;
use flag::Flag;
use hash_cache::HashCache;
use store::Store;

const FAIL: &'static [u8; 4] = &[102, 97, 105, 108];
const ALL_CACHE: &'static str = "$all_flags$";

#[derive(Debug)]
pub struct RedisStore<T> {
    key: String,
    client: Client,
    cache: HashCache<T>,
    all_cache: HashCache<HashMap<String, T>>,
    timeout: Duration,
}

pub type RedisStoreResult<T> = Result<T, BannerError>;

impl<T: Clone + FromRedisValue + ToRedisArgs> RedisStore<T> {
    pub fn open<S, U>(
        host: S,
        port: u32,
        prefix: Option<U>,
        timeout: Option<Duration>,
    ) -> RedisStoreResult<RedisStore<T>>
    where
        S: Into<String>,
        U: Into<String>,
    {
        RedisStore::open_with_url(format!("redis://{}:{}", host.into(), port), prefix, timeout)
    }

    pub fn open_with_url<S, U>(
        url: S,
        prefix: Option<U>,
        timeout: Option<Duration>,
    ) -> RedisStoreResult<RedisStore<T>>
    where
        S: Into<String>,
        U: Into<String>,
    {
        let client =
            Client::open(url.into().as_str()).map_err(|_| BannerError::InvalidRedisConfig)?;

        Ok(RedisStore::open_with_client(client, prefix, timeout))
    }

    pub fn open_with_client<S>(
        client: Client,
        prefix: Option<S>,
        timeout: Option<Duration>,
    ) -> RedisStore<T>
    where
        S: Into<String>,
    {
        let dur = timeout.unwrap_or(Duration::new(0, 0));

        RedisStore {
            key: RedisStore::<T>::features_key(prefix),
            client: client,
            cache: HashCache::new(dur),
            all_cache: HashCache::new(dur),
            timeout: dur,
        }
    }

    fn features_key<S>(prefix: Option<S>) -> String
    where
        S: Into<String>,
    {
        prefix.map(|p| p.into()).unwrap_or("banner".into()) + "::features"
    }

    fn conn(&self) -> RedisStoreResult<Connection> {
        // Get a single connection to group requests on
        self.client
            .get_connection()
            .map_err(BannerError::RedisFailure)
    }

    fn full_path(&self, path: &str) -> String {
        [self.key.as_str(), "::", path].concat()
    }

    fn full_key(&self, path: &str, key: &str) -> String {
        [self.key.as_str(), "::", path, "::", key].concat()
    }

    fn get_raw(&self, path: &str, key: &str, conn: &Connection) -> Option<T> {
        conn.hget(self.full_path(path), key.to_string()).ok()
    }

    fn put_raw(&self, path: &str, key: &str, item: &T, conn: &Connection) -> RedisStoreResult<()> {
        // Manually serialize to redis storable value to allow for failure handling
        let item_ser = item.to_redis_args();

        if item_ser[0].as_slice() != FAIL {
            let res: RedisResult<u8> = conn.hset(self.full_path(path), key.to_string(), item_ser);

            self.all_cache.remove(ALL_CACHE);
            self.cache.insert(self.full_key(path, key), item);

            res.map(|_| ()).map_err(BannerError::RedisFailure)
        } else {
            Err(BannerError::FailedToSerializeItem)
        }
    }

    fn delete_raw(&self, path: &str, key: &str, conn: &Connection) -> RedisStoreResult<()> {
        let res: RedisResult<u8> = conn.hdel(self.full_path(path), key.to_string());
        res.map(|_| ()).map_err(BannerError::RedisFailure)
    }

    fn start<S: FromRedisValue>(&self, path: &str, conn: &Connection) -> RedisStoreResult<()> {
        let res: RedisResult<S> = cmd("WATCH").arg(self.full_path(path)).query(conn);
        res.map(|_| ()).map_err(BannerError::RedisFailure)
    }

    fn cleanup<S: FromRedisValue>(&self, conn: &Connection) -> RedisStoreResult<()> {
        let res: RedisResult<S> = cmd("UNWATCH").query(conn);
        res.map(|_| ()).map_err(BannerError::RedisFailure)
    }
}

impl<T> Store for RedisStore<T>
where
    T: Clone + FromRedisValue + ToRedisArgs + Debug,
{
    type Item = T;
    type Error = BannerError;

    fn get(&self, path: &str, key: &str) -> Result<Option<T>, BannerError> {
        match self.cache.get(self.full_key(path, key).as_str()) {
            Ok(Some(item)) => Ok(Some(item)),
            _ => self.conn().map(|conn| {
                let item = self.get_raw(path, key, &conn);

                if let Some(ref val) = item {
                    self.cache.insert(self.full_key(path, key), val);
                }

                item
            }),
        }
    }

    fn get_all(&self, path: &str) -> Result<HashMap<String, T>, BannerError> {
        let r = self.all_cache
            .get(ALL_CACHE)
            .and_then(|map| map.ok_or(BannerError::AllCacheMissing))
            .or_else(|_| {
                self.conn()?
                    .hgetall(self.full_path(path))
                    .map(|map: HashMap<String, T>| {
                        self.all_cache.insert(ALL_CACHE, &map);
                        map
                    })
                    .map_err(BannerError::RedisFailure)
            });

        r
    }

    fn delete(&self, path: &str, key: &str) -> Result<Option<T>, BannerError> {
        // Ignores cache lookup
        let conn = self.conn()?;
        let _: () = self.start::<()>(path, &conn)?;

        let lookup = self.get(path, key);
        let store_res = self.delete_raw(path, key, &conn);
        self.cleanup::<()>(&conn);

        store_res.and_then(|_| {
            self.all_cache.clear();
            self.cache
                .remove(self.full_key(path, key).as_str())
                .and_then(|_| lookup)
        })
    }

    fn upsert(&self, path: &str, key: &str, item: &T) -> Result<Option<T>, BannerError> {
        // Ignores cache lookup
        let conn = self.conn()?;
        let _: () = self.start::<()>(path, &conn)?;

        let lookup = self.get(path, key);

        let store_res = self.put_raw(path, key, item, &conn);
        self.cleanup::<()>(&conn);

        store_res.and_then(|_| {
            self.all_cache.clear();
            self.cache
                .insert(self.full_key(path, key), item)
                .and_then(|_| lookup)
        })
    }
}

impl FromRedisValue for Flag {
    fn from_redis_value(v: &RedisValue) -> RedisResult<Flag> {
        match *v {
            RedisValue::Data(ref data) => {
                let data = String::from_utf8(data.clone());

                data.or_else(|_| Err((ErrorKind::TypeError, "Expected utf8 string").into()))
                    .and_then(|ser| {
                        serde_json::from_str(ser.as_str()).or_else(|_| {
                            let err = (ErrorKind::TypeError, "Unable to deserialize json to Flag");
                            Err(err.into())
                        })
                    })
            }
            ref x => {
                let err = (
                    ErrorKind::TypeError,
                    "Recieved non-data type for deserializing",
                );
                Err(err.into())
            }
        }
    }
}

impl<'a> ToRedisArgs for Flag {
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        let ser = serde_json::to_string(&self);

        vec![
            match ser {
                Ok(json) => json.as_bytes().into(),

                // Because this trait can not normally fail, but json serialization
                // can fail, the failure cause is encoded as a special value that
                // is checked by the store
                Err(_) => "fail".to_string().as_bytes().into(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use flag::*;
    use store::*;

    use super::*;

    const PATH: &'static str = "app::env";

    fn f<S: Into<String>>(key: S, enabled: bool) -> Flag {
        Flag::new(key, "app", "env", FlagValue::Bool(true), 1, enabled)
    }

    fn dataset(p: &str, dur: u64) -> RedisStore<Flag> {
        let store =
            RedisStore::open("0.0.0.0", 6379, Some(p), Some(Duration::new(dur, 0))).unwrap();
        let flags = vec![f("f1", false), f("f2", true)];

        for flag in flags.into_iter() {
            store.upsert(PATH, flag.key(), &flag);
        }

        store
    }

    #[test]
    fn test_gets_items() {
        let data = dataset("get_items", 0);

        assert_eq!(data.get(PATH, "f1").unwrap().unwrap(), f("f1", false));
        assert_eq!(data.get(PATH, "f2").unwrap().unwrap(), f("f2", true));
        assert!(data.get(PATH, "f3").unwrap().is_none());
    }

    #[test]
    fn test_gets_all_items() {
        let mut test_map = HashMap::new();
        test_map.insert("f1", f("f1", false));
        test_map.insert("f2", f("f2", true));

        let res = dataset("all_items", 0).get_all(PATH);

        assert!(res.is_ok());

        let map = res.unwrap();
        assert_eq!(map.len(), test_map.len());
        assert_eq!(map.get("f1").unwrap(), test_map.get("f1").unwrap());
        assert_eq!(map.get("f2").unwrap(), test_map.get("f2").unwrap());
    }

    #[test]
    fn test_deletes_without_cache() {
        let data = dataset("delete_no_cache", 0);

        assert_eq!(data.get_all(PATH).unwrap().len(), 2);

        // Test flag #1
        let f1 = data.delete(PATH, "f1");
        assert_eq!(f1.unwrap().unwrap(), f("f1", false));

        let f1_2 = data.get(PATH, "f1");
        assert!(f1_2.unwrap().is_none());

        // Test flag #2
        let f2 = data.delete(PATH, "f2");
        assert_eq!(f2.unwrap().unwrap(), f("f2", true));

        let f2_2 = data.get(PATH, "f2");
        assert!(f2_2.unwrap().is_none());

        assert_eq!(data.get_all(PATH).unwrap().len(), 0);
    }

    #[test]
    fn test_deletes_with_cache() {
        let data = dataset("delete_cache", 30);

        assert_eq!(data.get_all(PATH).unwrap().len(), 2);

        // Test flag #1
        let f1 = data.delete(PATH, "f1");
        assert_eq!(f1.unwrap().unwrap(), f("f1", false));

        let f1_2 = data.get(PATH, "f1");
        assert!(f1_2.unwrap().is_none());

        // Test flag #2
        let f2 = data.delete(PATH, "f2");
        assert_eq!(f2.unwrap().unwrap(), f("f2", true));

        let f2_2 = data.get(PATH, "f2");
        assert!(f2_2.unwrap().is_none());

        assert_eq!(data.get_all(PATH).unwrap().len(), 0);
    }

    #[test]
    fn test_replacements_without_cache() {
        let data = dataset("replace_no_cache", 0);

        assert_eq!(data.get_all(PATH).unwrap().len(), 2);

        // Test flag #1
        let f1 = data.upsert(PATH, "f1", &f("f1", true));
        assert_eq!(f1.unwrap().unwrap(), f("f1", false));

        let f1_2 = data.get(PATH, "f1");
        assert_eq!(f1_2.unwrap().unwrap(), f("f1", true));

        assert_eq!(data.get_all(PATH).unwrap().len(), 2);
    }

    #[test]
    fn test_replacements_with_cache() {
        let data = dataset("replace_cache", 30);

        assert_eq!(data.get_all(PATH).unwrap().len(), 2);

        // Test flag #1
        let f1 = data.upsert(PATH, "f1", &f("f1", true));
        assert_eq!(f1.unwrap().unwrap(), f("f1", false));

        let f1_2 = data.get(PATH, "f1");
        assert_eq!(f1_2.unwrap().unwrap(), f("f1", true));

        assert_eq!(data.get_all(PATH).unwrap().len(), 2);
    }
}
