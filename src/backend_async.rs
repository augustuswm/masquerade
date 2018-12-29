use futures::future::Either;
use futures::{future, Future, Stream};
use hashbrown::HashMap;
use log::{debug, info, warn};
use redis_async::client::paired::{paired_connect, PairedConnection};
use redis_async::client::pubsub::{pubsub_connect, PubsubStream};
use redis_async::resp::{FromResp, RespValue};
use redis_async::resp_array;

use std::borrow::Cow;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use crate::error::Error;
use crate::hash_cache::HashCache;
use crate::redis::RedisHashMap;

pub const FAIL: &'static [u8; 4] = &[102, 97, 105, 108];
const ALL_CACHE: &'static str = ":all_flags$";

#[derive(Debug)]
pub struct AsyncRedisStore<P, T> {
    key: String,
    address: SocketAddr,
    topic: Arc<String>,
    cache: HashCache<T>,
    all_cache: HashCache<HashMap<String, T>>,
    timeout: Duration,
    _key: ::std::marker::PhantomData<P>,
}

fn get_raw<T>(
    conn: &PairedConnection,
    path: &str,
    key: &str,
) -> impl Future<Item = Option<T>, Error = Error>
where
    T: FromResp,
{
    if key == "" {
        Either::A(future::err(Error::EmptyKey))
    } else {
        Either::B(conn.send(resp_array!["HGET", path, key]).map_err(fail))
    }
}

fn fail<T>(err: T) -> Error
where
    T: Into<Error>,
    T: Debug,
{
    warn!("Operation failed: {:?}", err);
    err.into()
}

fn send_notification(
    conn: &PairedConnection,
    topic: &str,
    key: &str,
) -> impl Future<Item = (), Error = Error> {
    conn.send::<i32>(resp_array!["PUBLISH", topic, key])
        .map(move |_| ())
        .map_err(fail)
}

impl<P, T> AsyncRedisStore<P, T>
where
    P: Clone + AsRef<str>,
    T: Clone + FromResp + Into<RespValue>,
{
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
            topic: Arc::new(topic.into()),
            cache: HashCache::new(dur),
            all_cache: HashCache::new(dur),
            timeout: dur,
            _key: ::std::marker::PhantomData,
        }
    }

    fn features_key<S>(prefix: Option<S>) -> String
    where
        S: Into<String>,
    {
        prefix.map(|p| p.into()).unwrap_or("masquerade".into()) + ":flags"
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

    pub fn notify(&self, path: &P, key: &str) -> impl Future<Item = (), Error = Error> {
        let all_key = [path.as_ref(), ALL_CACHE].concat();
        let item_key = self.full_key(path, key);
        let topic = self.topic.clone();

        debug!("Creating notification for: {} {}", topic, item_key);
        debug!("Creating notification for: {} {}", topic, all_key);

        self.conn().and_then(move |conn| {
            send_notification(&conn, topic.as_ref(), &all_key)
                .join(send_notification(&conn, topic.as_ref(), &item_key))
                .map(|_| ())
        })
    }

    pub fn get<S>(&self, path: P, key: S) -> impl Future<Item = Option<T>, Error = Error>
    where
        S: Into<String>,
    {
        let key = key.into();
        let full_path = self.full_path(&path);
        let full_key = self.full_key(&path, key.as_ref());
        let cache = self.cache.clone();

        debug!("Perform get: {}", full_key);

        match cache.get(full_key.as_str()) {
            Some(item) => {
                debug!("Cache hit: {}", full_key);
                Either::A(future::ok(Some(item)))
            }
            None => {
                debug!("Cache miss: {}", full_key);
                Either::B(self.conn().and_then(move |conn| {
                    get_raw(&conn, &full_path, key.as_ref()).map(move |resp| {
                        if let Some(ref val) = resp {
                            debug!("Get found element: {}", full_key);
                            let _ = cache.insert(full_key, val);
                        };

                        resp
                    })
                }))
            }
        }
    }

    pub fn get_all(&self, path: P) -> impl Future<Item = HashMap<String, T>, Error = Error> {
        let key = [path.as_ref(), ALL_CACHE].concat();
        let full_path = self.full_path(&path);
        let all_cache = self.all_cache.clone();

        debug!("Perform get all: {}", key);

        match all_cache.get(key.as_str()) {
            Some(map) => {
                debug!("Cache hit: {}", key);
                Either::A(future::ok(map))
            }
            None => {
                debug!("Cache miss: {}", key);
                Either::B(self.conn().and_then(move |conn| {
                    conn.send::<RedisHashMap<String, T>>(resp_array!["HGETALL", full_path])
                        .map(move |resp| {
                            let map = resp.into_hashmap();
                            debug!("Getall found {} elements", map.len());
                            let _ = all_cache.insert(key.as_str(), &map);
                            map
                        })
                        .map_err(fail)
                }))
            }
        }
    }

    pub fn delete<S>(&self, path: P, key: S) -> impl Future<Item = Option<T>, Error = Error>
    where
        S: Into<String>,
    {
        let key = key.into();
        let full_key = self.full_key(&path, key.as_ref());
        let full_path = self.full_path(&path);
        let all_cache = self.all_cache.clone();
        let cache = self.cache.clone();
        let notification = self.notify(&path, &key);

        debug!("Perform delete: {}", full_key);

        self.conn().and_then(move |conn| {
            get_raw(&conn, &full_path, key.as_ref()).and_then(move |resp| {
                conn.send::<i32>(resp_array!["HDEL", &full_path, key])
                    .map_err(|err| err.into())
                    .and_then(move |_| {
                        let _ = all_cache.clear();
                        let _ = cache.remove(full_key.as_str());
                        notification.map(|_| resp)
                    })
            })
        })
    }

    pub fn upsert<S>(
        &self,
        path: P,
        key: S,
        item: &T,
    ) -> impl Future<Item = Option<T>, Error = Error>
    where
        S: Into<String>,
    {
        let key = key.into();
        let full_key = self.full_key(&path, key.as_ref());
        let full_path = self.full_path(&path);
        let item = item.to_owned();
        let all_cache = self.all_cache.clone();
        let cache = self.cache.clone();
        let notification = self.notify(&path, &key);

        let ser: RespValue = item.clone().into();

        if ser == RespValue::BulkString(FAIL.to_vec()) {
            return Either::A(future::err(Error::FailedToSerializeItem));
        }

        debug!("Perform upsert: {}", full_key);

        Either::B(self.conn().and_then(move |conn| {
            get_raw(&conn, &full_path, key.as_ref()).and_then(move |resp| {
                conn.send::<i32>(resp_array!["HSET", &full_path, key, item.clone().into()])
                    .map_err(fail)
                    .and_then(move |_| {
                        let _ = all_cache.clear();
                        let _ = cache.remove(full_key.as_str());
                        notification.map(|_| resp)
                    })
            })
        }))
    }

    pub fn update_sub(
        &self,
    ) -> impl Future<Item = impl Stream<Item = (), Error = Error>, Error = Error> {
        self.stream_conn().map(|stream| {
            stream
                .map(|_| ())
                .map_err(|_| Error::RedisAsyncSubMessageFailure)
        })
    }

    pub fn updater(&self) -> impl Future<Item = (), Error = ()> {
        let cache = self.cache.clone();
        let all_cache = self.all_cache.clone();

        self.stream_conn().map_err(|_| ()).and_then(move |stream| {
            stream
                .for_each(move |msg| {
                    let key = match msg {
                        RespValue::BulkString(ref bytes) => String::from_utf8_lossy(bytes),
                        RespValue::SimpleString(ref string) => Cow::Borrowed(string.as_str()),
                        _ => Cow::Borrowed(""),
                    };

                    info!("Update for {:?}", key);
                    let _ = all_cache.remove(key.as_ref());
                    let _ = cache.remove(key.as_ref());
                    info!("Cleared cache for {:?}", key);
                    future::ok(())
                })
                .map_err(|_| ())
        })
    }
}

#[cfg(test)]
mod tests {
    use futures::future::ok;
    use futures::Future;
    use tokio::runtime::current_thread::Runtime;
    use tokio::timer::{Interval, Timeout};

    use crate::error::Error;
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
            let _ = run(store.upsert(path(), flag.key(), &flag));
        }

        store
    }

    fn run<F>(to_run: F) -> F::Item
    where
        F: Future<Error = Error>,
    {
        run_to_result(to_run)
            .map_err(|err| println!("Failed to run test future: {:?}", err))
            .unwrap()
    }

    fn run_to_result<F>(to_run: F) -> Result<F::Item, F::Error>
    where
        F: Future<Error = Error>,
    {
        Runtime::new().unwrap().block_on(to_run)
    }

    #[test]
    fn test_gets_items() {
        let data = dataset("get_items", 0);

        assert_eq!(run(data.get(path(), "f1")).unwrap(), f("f1", false));
        assert_eq!(run(data.get(path(), "f2")).unwrap(), f("f2", true));
        assert!(run(data.get(path(), "f3")).is_none());
    }

    #[test]
    fn test_gets_all_items() {
        let mut test_map = HashMap::new();
        test_map.insert("f1", f("f1", false));
        test_map.insert("f2", f("f2", true));
        let dataset = dataset("all_items", 0);

        let map = run(dataset.get_all(path()));

        assert_eq!(map.len(), test_map.len());
        assert_eq!(map.get("f1").unwrap(), test_map.get("f1").unwrap());
        assert_eq!(map.get("f2").unwrap(), test_map.get("f2").unwrap());
    }

    #[test]
    fn test_deletes_without_cache() {
        let data = dataset("delete_no_cache", 0);

        assert_eq!(run(data.get_all(path())).len(), 2);

        // Test flag #1
        let f1 = run(data.delete(path(), "f1"));
        assert_eq!(f1.unwrap(), f("f1", false));

        let f1_2 = run(data.get(path(), "f1"));
        assert!(f1_2.is_none());

        // Test flag #2
        let f2 = run(data.delete(path(), "f2"));
        assert_eq!(f2.unwrap(), f("f2", true));

        let f2_2 = run(data.get(path(), "f2"));
        assert!(f2_2.is_none());

        assert_eq!(run(data.get_all(path())).len(), 0);
    }

    #[test]
    fn test_deletes_with_cache() {
        let data = dataset("delete_cache", 30);

        assert_eq!(run(data.get_all(path())).len(), 2);

        // Test flag #1
        let f1 = run(data.delete(path(), "f1"));
        assert_eq!(f1.unwrap(), f("f1", false));

        let f1_2 = run(data.get(path(), "f1"));
        assert!(f1_2.is_none());

        // Test flag #2
        let f2 = run(data.delete(path(), "f2"));
        assert_eq!(f2.unwrap(), f("f2", true));

        let f2_2 = run(data.get(path(), "f2"));
        assert!(f2_2.is_none());

        assert_eq!(run(data.get_all(path())).len(), 0);
    }

    #[test]
    fn test_replacements_without_cache() {
        let data = dataset("replace_no_cache", 0);

        assert_eq!(run(data.get_all(path())).len(), 2);

        // Test flag #1
        let f1 = run(data.upsert(path(), "f1", &f("f1", true)));
        assert_eq!(f1.unwrap(), f("f1", false));

        let f1_2 = run(data.get(path(), "f1"));
        assert_eq!(f1_2.unwrap(), f("f1", true));

        assert_eq!(run(data.get_all(path())).len(), 2);
    }

    #[test]
    fn test_replacements_with_cache() {
        let data = dataset("replace_cache", 30);

        assert_eq!(run(data.get_all(path())).len(), 2);

        // Test flag #1
        let f1 = run(data.upsert(path(), "f1", &f("f1", true)));
        assert_eq!(f1.unwrap(), f("f1", false));

        let f1_2 = run(data.get(path(), "f1"));
        assert_eq!(f1_2.unwrap(), f("f1", true));

        assert_eq!(run(data.get_all(path())).len(), 2);
    }

    #[test]
    fn test_subscribes_and_notifies() {
        let data = dataset("pub_sub", 0);

        let mut runner = Runtime::new().unwrap();

        // FIXME: I'm sure there is a better way to test this.
        // Can not figure it out currently.

        let sub = Timeout::new(
            data.update_sub()
                .map_err(|_| ())
                .and_then(|sub_conn| sub_conn.take(1).for_each(|v| ok(v)).map_err(|_| ())),
            Duration::new(2, 0),
        );

        let notifier = Interval::new_interval(Duration::new(1, 0))
            .map_err(|_| ())
            .for_each(move |_| data.notify(&path(), "f1").map_err(|_| ()))
            .map(|_| ());

        runner.spawn(notifier);

        let res = runner.block_on(sub).unwrap();

        assert_eq!((), res);
    }

    fn check_for_empty_key_error<F, T>(to_run: F)
    where
        F: Future<Item = T, Error = Error>,
        T: Debug,
    {
        let result = run_to_result(to_run);

        assert!(match result.unwrap_err() {
            Error::EmptyKey => true,
            _ => false,
        });
    }

    #[test]
    fn test_empty_key_failures() {
        let data = dataset("empty_keys", 0);

        check_for_empty_key_error(data.get(path(), ""));

        check_for_empty_key_error(data.upsert(path(), "", &f("f1", true)));

        check_for_empty_key_error(data.delete(path(), ""));
    }
}
