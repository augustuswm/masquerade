use futures::{future, Future, Stream};
use futures::future::Either;
use log::{debug, info};
use redis_async::resp_array;
use redis_async::client::paired::{paired_connect, PairedConnection};
use redis_async::client::pubsub::{pubsub_connect, PubsubStream};
use redis_async::resp::{FromResp, RespValue};

use std::collections::HashMap;
use std::net::SocketAddr;

use std::time::{Duration};

use crate::error::Error;
use crate::hash_cache::HashCache;

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

    fn conn(&self) -> impl Future<Item = PairedConnection, Error = Error> {
        paired_connect(&self.address).map_err(|err| err.into())
    }

    fn stream_conn(&self) -> impl Future<Item = PubsubStream, Error = Error> {
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

    pub fn notify(&self, path: &P) -> impl Future<Item = (), Error = Error> {
        let key = [path.as_ref(), ALL_CACHE].concat();
        let topic = self.topic.clone();

        self.conn().and_then(|conn| {
            conn.send::<i32>(resp_array!["PUBLISH", topic, key]).map(|_| {
                ()
            }).map_err(|err| err.into())
        })
    }

    pub fn get(&self, path: &P, key: &str) -> impl Future<Item = Option<T>, Error = Error> {
        let key = key.to_string();
        let full_path = self.full_path(path);
        let full_key = self.full_key(&path, &key);
        let cache = self.cache.clone();

        match cache.get(full_key.as_str()) {
            Ok(Some(item)) => {
                debug!("Cache hit: {}", full_key);
                Either::A(future::ok(Some(item)))
            },
            _ => {
                debug!("Cache miss: {}", full_key);
                Either::B(self.conn().and_then(move |conn| {
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
    }

    pub fn get_all(&self, path: &P) -> impl Future<Item = HashMap<String, T>, Error = Error> {
        let key = [path.as_ref(), ALL_CACHE].concat();
        let full_path = self.full_path(path);
        let all_cache = self.all_cache.clone();

        match all_cache.get(key.as_str()) {
            Ok(Some(map)) => {
                debug!("Cache hit: {}", key);
                Either::A(future::ok(map))
            }
            _ => {
                debug!("Cache miss: {}", key);
                Either::B(self.conn().and_then(move |conn| {
                  conn.send::<HashMap<String, T>>(resp_array!["HGETALL", full_path])
                    .map(move |resp| {
                        let _ = all_cache.insert(key.as_str(), &resp);
                        resp
                    })
                    .map_err(|err| err.into())
                }))
            }
        }
    }

    pub fn delete(&self, path: &P, key: &str) -> impl Future<Item = Option<T>, Error = Error> {
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

    pub fn upsert(&self, path: &P, key: &str, item: &T) -> impl Future<Item = Option<T>, Error = Error> {
        let key = key.to_string();
        let path = path.clone();
        let full_key = self.full_key(&path, &key);
        let full_path = self.full_path(&path);
        let item = item.to_owned();
        let all_cache = self.all_cache.clone();
        let cache = self.cache.clone();
        let notification = self.notify(&path);

        let ser: RespValue = item.clone().into();

        if ser == RespValue::BulkString(FAIL.to_vec()) {
            return Either::A(future::err(Error::FailedToSerializeItem))
        }

        Either::B(self.conn().and_then(move |conn| {
            conn.send::<Option<T>>(resp_array!["HGET", &full_path, &key])
                .map_err(|err| {
                    err.into()
                })
                .and_then(move |resp| {
                    conn.send::<i32>(resp_array!["HSET", &full_path, &key, item.clone().into()])
                        .map_err(|err| err.into())
                        .and_then(move |_| {
                            let _ = all_cache.clear();
                            let _ = cache.insert(full_key, &item);
                            notification.map(|_| resp)
                        })
                })
        }))
    }

    pub fn update_sub(&self) -> impl Future<Item = impl Stream<Item = (), Error = Error>, Error = Error> {
        self.stream_conn().map(|stream| {
            stream.map(|_| ()).map_err(|_| Error::RedisAsyncSubMessageFailure)
        })
    }

    pub fn updater(&self) -> impl Future<Item = (), Error = ()> {
        let all_cache = self.all_cache.clone();

        self.stream_conn()
            .map_err(|_| ())
            .and_then(move |stream| {
                stream.for_each(move |msg| {
                    info!("Update for {:?}", msg);
                    let _ = all_cache.clear();
                    info!("Cleared cache for {:?}", msg);
                    future::ok(())
                }).map_err(|_| ())
            })
    }
}

macro_rules! redis_conversions {
    ($struct:ident) => {
        impl FromResp for $struct {
            fn from_resp_int(resp: RespValue) -> Result<$struct, RedisAsyncError> {
                match resp {
                    RespValue::BulkString(ref bytes) => {
                        serde_json::from_str(&String::from_utf8_lossy(bytes)).or_else(|_| {
                            Err(redis_async::error::resp("Cannot convert into a $struct", redis_async::resp::RespValue::BulkString(bytes.to_owned())))
                        })
                    },
                    RespValue::SimpleString(ref string) => {
                        serde_json::from_str(string.as_str()).or_else(|_| {
                            Err(redis_async::error::resp("Cannot convert into a $struct", resp.to_owned()))
                        })
                    },
                    _ => Err(redis_async::error::resp("Cannot convert into a $struct", resp)),
                }
            }
        }

        impl Into<RespValue> for $struct {
            fn into(self: Self) -> RespValue {
                let res = serde_json::to_string(&self);

                match res {
                    Ok(ser) => RespValue::BulkString(ser.as_bytes().to_vec()),
                    Err(_) => RespValue::BulkString("fail".as_bytes().to_vec())
                }
                
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::{Future};
    use futures::future::ok;
    use tokio::runtime::current_thread::Runtime;
    use tokio::timer::{Interval, Timeout};

    use crate::flag::*;

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
        let prefix = "async_".to_string() + p;
        let topic = "async_masquerade_".to_string() + p;

        let store = AsyncRedisStore::open(addr, topic, Some(prefix), Some(Duration::new(dur, 0)));
        let flags = vec![f("f1", false), f("f2", true)];

        for flag in flags.into_iter() {
            let _ = run(store.upsert(&path(), flag.key(), &flag));
        }

        store
    }

    fn run<F>(to_run: F) -> F::Item where F: Future {
        Runtime::new().unwrap().block_on(to_run).map_err(|_| ()).unwrap()
    }

    #[test]
    fn test_gets_items() {
        let data = dataset("get_items", 0);

        assert_eq!(run(data.get(&path(), "f1")).unwrap(), f("f1", false));
        assert_eq!(run(data.get(&path(), "f2")).unwrap(), f("f2", true));
        assert!(run(data.get(&path(), "f3")).is_none());
    }

    #[test]
    fn test_gets_all_items() {
        let mut test_map = HashMap::new();
        test_map.insert("f1", f("f1", false));
        test_map.insert("f2", f("f2", true));
        let dataset = dataset("all_items", 0);

        let map = run(dataset.get_all(&path()));

        assert_eq!(map.len(), test_map.len());
        assert_eq!(map.get("f1").unwrap(), test_map.get("f1").unwrap());
        assert_eq!(map.get("f2").unwrap(), test_map.get("f2").unwrap());
    }

    #[test]
    fn test_deletes_without_cache() {
        let data = dataset("delete_no_cache", 0);

        assert_eq!(run(data.get_all(&path())).len(), 2);

        // Test flag #1
        let f1 = run(data.delete(&path(), "f1"));
        assert_eq!(f1.unwrap(), f("f1", false));

        let f1_2 = run(data.get(&path(), "f1"));
        assert!(f1_2.is_none());

        // Test flag #2
        let f2 = run(data.delete(&path(), "f2"));
        assert_eq!(f2.unwrap(), f("f2", true));

        let f2_2 = run(data.get(&path(), "f2"));
        assert!(f2_2.is_none());

        assert_eq!(run(data.get_all(&path())).len(), 0);
    }

    #[test]
    fn test_deletes_with_cache() {
        let data = dataset("delete_cache", 30);

        assert_eq!(run(data.get_all(&path())).len(), 2);

        // Test flag #1
        let f1 = run(data.delete(&path(), "f1"));
        assert_eq!(f1.unwrap(), f("f1", false));

        let f1_2 = run(data.get(&path(), "f1"));
        assert!(f1_2.is_none());

        // Test flag #2
        let f2 = run(data.delete(&path(), "f2"));
        assert_eq!(f2.unwrap(), f("f2", true));

        let f2_2 = run(data.get(&path(), "f2"));
        assert!(f2_2.is_none());

        assert_eq!(run(data.get_all(&path())).len(), 0);
    }

    #[test]
    fn test_replacements_without_cache() {
        let data = dataset("replace_no_cache", 0);

        assert_eq!(run(data.get_all(&path())).len(), 2);

        // Test flag #1
        let f1 = run(data.upsert(&path(), "f1", &f("f1", true)));
        assert_eq!(f1.unwrap(), f("f1", false));

        let f1_2 = run(data.get(&path(), "f1"));
        assert_eq!(f1_2.unwrap(), f("f1", true));

        assert_eq!(run(data.get_all(&path())).len(), 2);
    }

    #[test]
    fn test_replacements_with_cache() {
        let data = dataset("replace_cache", 30);

        assert_eq!(run(data.get_all(&path())).len(), 2);

        // Test flag #1
        let f1 = run(data.upsert(&path(), "f1", &f("f1", true)));
        assert_eq!(f1.unwrap(), f("f1", false));

        let f1_2 = run(data.get(&path(), "f1"));
        assert_eq!(f1_2.unwrap(), f("f1", true));

        assert_eq!(run(data.get_all(&path())).len(), 2);
    }

    #[test]
    fn test_subscribes_and_notifies() {
        let data = dataset("pub_sub", 0);

        let mut runner = Runtime::new().unwrap();

        // FIXME: I'm sure there is a better way to test this.
        // Can not figure it out currently. 

        let sub = Timeout::new(data.update_sub()
            .map_err(|_| ())
            .and_then(|sub_conn| {
                sub_conn.take(1).for_each(|v| {
                    ok(v)
                }).map_err(|_| ())
            }), Duration::new(2, 0));

        let notifier = Interval::new_interval(Duration::new(1, 0))
            .map_err(|_| ())
            .for_each(move |_| data.notify(&path()).map_err(|_| ()))
            .map(|_| ());

        runner.spawn(notifier);

        let res = runner.block_on(sub).unwrap();

        assert_eq!((), res);
    }
}
