use futures::{future, Future, Stream};
use futures::future::Either;
use redis_async::client::paired::{paired_connect, PairedConnection};
use redis_async::client::pubsub::{pubsub_connect, PubsubStream};
use redis_async::error::Error as AsyncRedisError;
use redis_async::resp::{FromResp, RespValue};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use error::BannerError;
use hash_cache::HashCache;

const FAIL: &'static [u8; 4] = &[102, 97, 105, 108];
const ALL_CACHE: &'static str = ":all_flags$";

#[derive(Debug)]
pub struct AsyncRedisStore<P, T> {
    key: String,
    address: SocketAddr,
    topic: String,
    cache: HashCache<T>,
    all_cache: HashCache<HashMap<String, T>>,
    timeout: Duration,
    _key: ::std::marker::PhantomData<P>,
}

impl<P, T> AsyncRedisStore<P, T> where P: Clone + AsRef<str>, T: Clone + FromResp + Into<RespValue> {
    pub fn open<S, U>(
        address: SocketAddr,
        topic: S,
        prefix: Option<U>,
        timeout: Option<Duration>,
    ) -> AsyncRedisStore<P, T>
    where
        U: Into<String>,
        S: Into<String>,
    {
        let dur = timeout.unwrap_or(Duration::new(0, 0));

        AsyncRedisStore {
            key: AsyncRedisStore::<P, T>::features_key(prefix),
            address: address,
            topic: topic.into(),
            cache: HashCache::new(dur),
            all_cache: HashCache::new(dur),
            timeout: dur,
            _key: ::std::marker::PhantomData
        }
    }

    fn features_key<S>(prefix: Option<S>) -> String
    where
        S: Into<String>,
    {
        prefix.map(|p| p.into()).unwrap_or("banner".into()) + ":features"
    }

    // fn conn(&self) -> AsyncRedisStoreResult<Connection> {
    fn conn(&self) -> impl Future<Item = PairedConnection, Error = BannerError> {
        paired_connect(&self.address).map_err(|err| err.into())
    }

    fn stream_conn(&self) -> impl Future<Item = PubsubStream, Error = BannerError> {
        let topic = self.topic.clone();

        pubsub_connect(&self.address)
            .and_then(move |conn| conn.subscribe(&topic))
            .map_err(|err| err.into())
    }

    fn full_path(&self, path: &P) -> String {
        [self.key.as_str(), ":", path.as_ref()].concat()
    }

    fn full_key(&self, path: &P, key: &str) -> String {
        [self.key.as_str(), ":", path.as_ref(), "/", key].concat()
    }

    pub fn notify(&self, _path: &P) -> impl Future<Item = (), Error = BannerError> {
        let topic = self.topic.clone();

        self.conn().and_then(|conn| {
            conn.send::<i32>(resp_array!["PUBLISH", topic, "update"]).map(|_| ()).map_err(|err| err.into())
        })
    }

    pub fn get(&self, path: &P, key: &str) -> impl Future<Item = Option<T>, Error = BannerError> {
        let key = key.to_string();
        let full_path = self.full_path(path);
        let full_key = self.full_key(&path, &key);
        let cache = self.cache.clone();

        match cache.get(self.full_key(&path, &key).as_str()) {
            Ok(Some(item)) => Either::A(future::ok(Some(item))),
            _ => Either::B(self.conn().and_then(move |conn| {
                  conn.send(resp_array!["HGET", full_path, &key])
                    .map(move |resp| {
                        if let Some(ref val) = resp {
                            let _ = cache.insert(full_key, val);
                        };

                        resp
                    })
                    .map_err(|err| err.into())
                }))
        }
    }

    pub fn get_all(&self, path: &P) -> impl Future<Item = HashMap<String, T>, Error = BannerError> {
        let key = [path.as_ref(), ALL_CACHE].concat();
        let full_path = self.full_path(path);
        let all_cache = self.all_cache.clone();

        match all_cache.get(key.as_str()) {
            Ok(Some(map)) => Either::A(future::ok(map)),
            _ => Either::B(self.conn().and_then(move |conn| {
                  conn.send::<HashMap<String, T>>(resp_array!["HGETALL", full_path])
                    .map(move |resp| {
                        let _ = all_cache.insert(key.as_str(), &resp);
                        resp
                    })
                    .map_err(|err| err.into())
                }))
        }
    }

    pub fn delete(&self, path: &P, key: &str) -> impl Future<Item = Option<T>, Error = BannerError> {
        let key = key.to_string();
        let path = path.clone();
        let full_key = self.full_key(&path, &key);
        let full_path = self.full_path(&path);
        let all_cache = self.all_cache.clone();
        let cache = self.cache.clone();
        let notification = self.notify(&path);

        self.conn().and_then(move |conn| {
            conn.send::<Option<T>>(resp_array!["HGET", &full_path, &key])
                .map_err(|err| err.into())
                .and_then(move |resp| {
                    conn.send::<i32>(resp_array!["HDEL", &full_path, &key])
                        .map_err(|err| err.into())
                        .and_then(move |_| {
                            let _ = all_cache.clear();
                            let _ = cache.remove(full_key.as_str());
                            notification.map(|_| resp)
                        })
                })
        })
    }

    pub fn upsert(&self, path: &P, key: &str, item: &T) -> impl Future<Item = Option<T>, Error = BannerError> {
        let key = key.to_string();
        let path = path.clone();
        let full_key = self.full_key(&path, &key);
        let full_path = self.full_path(&path);
        let item = item.to_owned();
        let all_cache = self.all_cache.clone();
        let cache = self.cache.clone();
        let notification = self.notify(&path);

        self.conn().and_then(move |conn| {
            conn.send::<Option<T>>(resp_array!["HGET", &full_path, &key])
                .map_err(|err| err.into())
                .and_then(move |resp| {
                    conn.send::<i32>(resp_array!["HSET", &full_path, &key, item.clone()])
                        .map_err(|err| err.into())
                        .and_then(move |_| {
                            let _ = all_cache.clear();
                            let _ = cache.insert(full_key, &item);
                            notification.map(|_| resp)
                        })
                })
        })
    }

    pub fn update_sub(&self) -> impl Future<Item = impl Stream<Item = (), Error = BannerError>, Error = BannerError> {
        self.stream_conn().map(|stream| {
            stream.map(|_| ()).map_err(|_| BannerError::RedisAsyncSubMessageFailure)
        })
    }
}

#[cfg(test)]
mod tests {
    use flag::*;

    use super::*;

    const PATH: &'static str = "the-owner-uuid-value:app:env";

    fn f<S: Into<String>>(key: S, enabled: bool) -> Flag {
        Flag::new(key, FlagValue::Bool(true), 1, enabled)
    }

    fn path() -> FlagPath {
        PATH.parse::<FlagPath>().unwrap()
    }

    fn dataset(p: &str, dur: u64) -> AsyncRedisStore<FlagPath, Flag> {
        let addr = "127.0.0.1:6379".to_string().parse().unwrap();

        let store = AsyncRedisStore::open(addr, "masquerade", Some(p), Some(Duration::new(dur, 0)));
        let flags = vec![f("f1", false), f("f2", true)];

        for flag in flags.into_iter() {
            let _ = store.upsert(&path(), flag.key(), &flag);
        }

        store
    }

    #[test]
    fn test_gets_items() {
        let data = dataset("get_items", 0);

        // assert_eq!(data.get(&path(), "f1").unwrap().unwrap(), f("f1", false));
        // assert_eq!(data.get(&path(), "f2").unwrap().unwrap(), f("f2", true));
        // assert!(data.get(&path(), "f3").unwrap().is_none());
    }

    #[test]
    fn test_gets_all_items() {
        let mut test_map = HashMap::new();
        test_map.insert("f1", f("f1", false));
        test_map.insert("f2", f("f2", true));
        let dataset = dataset("all_items", 0);

        let res = dataset.get_all(&path());

        // assert!(res.is_ok());

        // let map = res.unwrap();
        // assert_eq!(map.len(), test_map.len());
        // assert_eq!(map.get("f1").unwrap(), test_map.get("f1").unwrap());
        // assert_eq!(map.get("f2").unwrap(), test_map.get("f2").unwrap());
    }

    #[test]
    fn test_deletes_without_cache() {
        let data = dataset("delete_no_cache", 0);

        // assert_eq!(data.get_all(&path()).unwrap().len(), 2);

        // Test flag #1
        let f1 = data.delete(&path(), "f1");
        // assert_eq!(f1.unwrap().unwrap(), f("f1", false));

        let f1_2 = data.get(&path(), "f1");
        // assert!(f1_2.unwrap().is_none());

        // Test flag #2
        let f2 = data.delete(&path(), "f2");
        // assert_eq!(f2.unwrap().unwrap(), f("f2", true));

        let f2_2 = data.get(&path(), "f2");
        // assert!(f2_2.unwrap().is_none());

        // assert_eq!(data.get_all(&path()).unwrap().len(), 0);
    }

    #[test]
    fn test_deletes_with_cache() {
        let data = dataset("delete_cache", 30);

        // assert_eq!(data.get_all(&path()).unwrap().len(), 2);

        // Test flag #1
        let f1 = data.delete(&path(), "f1");
        // assert_eq!(f1.unwrap().unwrap(), f("f1", false));

        let f1_2 = data.get(&path(), "f1");
        // assert!(f1_2.unwrap().is_none());

        // Test flag #2
        let f2 = data.delete(&path(), "f2");
        // assert_eq!(f2.unwrap().unwrap(), f("f2", true));

        let f2_2 = data.get(&path(), "f2");
        // assert!(f2_2.unwrap().is_none());

        // assert_eq!(data.get_all(&path()).unwrap().len(), 0);
    }

    #[test]
    fn test_replacements_without_cache() {
        let data = dataset("replace_no_cache", 0);

        // assert_eq!(data.get_all(&path()).unwrap().len(), 2);

        // Test flag #1
        let f1 = data.upsert(&path(), "f1", &f("f1", true));
        // assert_eq!(f1.unwrap().unwrap(), f("f1", false));

        let f1_2 = data.get(&path(), "f1");
        // assert_eq!(f1_2.unwrap().unwrap(), f("f1", true));

        // assert_eq!(data.get_all(&path()).unwrap().len(), 2);
    }

    #[test]
    fn test_replacements_with_cache() {
        let data = dataset("replace_cache", 30);

        // assert_eq!(data.get_all(&path()).unwrap().len(), 2);

        // Test flag #1
        let f1 = data.upsert(&path(), "f1", &f("f1", true));
        // assert_eq!(f1.unwrap().unwrap(), f("f1", false));

        let f1_2 = data.get(&path(), "f1");
        // assert_eq!(f1_2.unwrap().unwrap(), f("f1", true));

        // assert_eq!(data.get_all(&path()).unwrap().len(), 2);
    }

    // TODO: Add subscription test

    // TODO: Add notify test
}
