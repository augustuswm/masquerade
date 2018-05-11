use futures::task::Task;
use redis::{cmd, Client, Commands, Connection, FromRedisValue, RedisResult, ToRedisArgs};

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use error::BannerError;
use hash_cache::HashCache;
use store::Store;

const FAIL: &'static [u8; 4] = &[102, 97, 105, 108];
const ALL_CACHE: &'static str = ":all_flags$";

#[derive(Debug)]
pub struct RedisStore<T> {
    key: String,
    client: Client,
    cache: HashCache<T>,
    all_cache: HashCache<HashMap<String, T>>,
    timeout: Duration,
    updated_at: Arc<RwLock<Instant>>,
    subs: Arc<RwLock<HashMap<String, Vec<(String, Option<Task>)>>>>,
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
            Client::open(url.into().as_ref()).map_err(|_| BannerError::InvalidRedisConfig)?;

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
            updated_at: Arc::new(RwLock::new(Instant::now())),
            subs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn features_key<S>(prefix: Option<S>) -> String
    where
        S: Into<String>,
    {
        prefix.map(|p| p.into()).unwrap_or("banner".into()) + ":features"
    }

    fn conn(&self) -> RedisStoreResult<Connection> {
        // Get a single connection to group requests on
        self.client
            .get_connection()
            .map_err(BannerError::RedisFailure)
    }

    fn full_path<P: AsRef<str>>(&self, path: &P) -> String {
        [self.key.as_str(), ":", path.as_ref()].concat()
    }

    fn full_key<P: AsRef<str>>(&self, path: &P, key: &str) -> String {
        [self.key.as_str(), ":", path.as_ref(), "/", key].concat()
    }

    fn get_raw<P: AsRef<str>>(&self, path: &P, key: &str, conn: &Connection) -> Option<T> {
        conn.hget(self.full_path(path), key.to_string()).ok()
    }

    fn put_raw<P: AsRef<str>>(
        &self,
        path: &P,
        key: &str,
        item: &T,
        conn: &Connection,
    ) -> RedisStoreResult<()> {
        // Manually serialize to redis storable value to allow for failure handling
        let item_ser = item.to_redis_args();

        if item_ser[0].as_slice() != FAIL {
            let res: RedisResult<u8> = conn.hset(self.full_path(path), key.to_string(), item_ser);
            res.map(|_| ()).map_err(BannerError::RedisFailure)
        } else {
            Err(BannerError::FailedToSerializeItem)
        }
    }

    fn delete_raw<P: AsRef<str>>(
        &self,
        path: &P,
        key: &str,
        conn: &Connection,
    ) -> RedisStoreResult<()> {
        let res: RedisResult<u8> = conn.hdel(self.full_path(path), key.to_string());
        res.map(|_| ()).map_err(BannerError::RedisFailure)
    }

    fn start<S: FromRedisValue, P: AsRef<str>>(
        &self,
        path: &P,
        conn: &Connection,
    ) -> RedisStoreResult<()> {
        let res: RedisResult<S> = cmd("WATCH").arg(self.full_path(path)).query(conn);
        res.map(|_| ()).map_err(BannerError::RedisFailure)
    }

    fn cleanup<S: FromRedisValue>(&self, conn: &Connection) -> RedisStoreResult<()> {
        let res: RedisResult<S> = cmd("UNWATCH").query(conn);
        res.map(|_| ()).map_err(BannerError::RedisFailure)
    }

    pub fn mark_updated(&self, time: Instant) -> bool {
        self.updated_at.write().map(|mut val| { *val = time; true }).unwrap_or(false)
    }

    pub fn notify<P>(&self, path: &P) -> usize where P: AsRef<str> {
        if let Ok(reader) = self.subs.read() {
            reader.get(path.as_ref().into()).map(|subs| {
                for &(_, ref task) in subs.iter() {
                    if let &Some(ref t) = task {
                        t.notify();
                    }
                };

                subs.len()
            }).unwrap_or(0)
        } else {
            0
        }
    }

    pub fn subs(&self) -> HashMap<String, usize> {
        let map = self.subs.read().unwrap();
        let mut ret_map = HashMap::new();

        for (k, v) in map.iter() {
            ret_map.insert(k.clone(), v.len());
        }

        ret_map
    }
}

impl<T, P> Store<P, T> for RedisStore<T>
where
    P: AsRef<str> + Debug,
    T: Clone + FromRedisValue + ToRedisArgs + Debug,
{
    type Error = BannerError;

    fn get(&self, path: &P, key: &str) -> Result<Option<T>, BannerError> {
        match self.cache.get(self.full_key(path, key).as_str()) {
            Ok(Some(item)) => Ok(Some(item)),
            _ => self.conn().map(|conn| {
                let item = self.get_raw(path, key, &conn);

                if let Some(ref val) = item {
                    let _ = self.cache.insert(self.full_key(path, key), val);
                }

                item
            }),
        }
    }

    fn get_all(&self, path: &P) -> Result<HashMap<String, T>, BannerError> {
        let key = [path.as_ref(), ALL_CACHE].concat();
        println!("{:?}", key);
        self.all_cache
            .get(key.as_str())
            .and_then(|map| map.ok_or(BannerError::AllCacheMissing))
            .or_else(|_| {
                self.conn()?
                    .hgetall(self.full_path(path))
                    .map(|map: HashMap<String, T>| {
                        let _ = self.all_cache.insert(key.as_str(), &map);
                        map
                    })
                    .map_err(BannerError::RedisFailure)
            })
    }

    fn delete(&self, path: &P, key: &str) -> Result<Option<T>, BannerError> {
        // Ignores cache lookup
        let conn = self.conn()?;
        let _: () = self.start::<(), P>(path, &conn)?;

        let lookup = self.get(path, key);
        let store_res = self.delete_raw(path, key, &conn);
        let _ = self.cleanup::<()>(&conn);

        let res = store_res.and_then(|_| {
            let _ = self.all_cache.clear();
            self.cache
                .remove(self.full_key(path, key).as_str())
                .and_then(|_| lookup)
        });

        self.mark_updated(Instant::now());
        self.notify(path);

        res
    }

    fn upsert(&self, path: &P, key: &str, item: &T) -> Result<Option<T>, BannerError> {
        // Ignores cache lookup
        let conn = self.conn()?;
        let _: () = self.start::<(), P>(path, &conn)?;

        let lookup = self.get(path, key);

        let store_res = self.put_raw(path, key, item, &conn);
        let _ = self.cleanup::<()>(&conn);

        let res = store_res.and_then(|_| {
            let _ = self.all_cache.clear();
            self.cache
                .insert(self.full_key(path, key), item)
                .and_then(|_| lookup)
        });

        self.mark_updated(Instant::now());
        self.notify(path);

        res
    }

    fn updated_at(&self) -> Result<Instant, BannerError> {
        self.updated_at.read().map(|val| *val).map_err(|_| BannerError::UpdatedAtPoisoned)
    }

    fn sub(&self, id: &str, path: &P, task: Option<Task>) -> bool {
        self.subs.write().map(|mut coll| {
            let subs = coll.entry(path.as_ref().into()).or_insert(vec![]);
            subs.push((id.to_string(), task))
        }).map(|_| true).unwrap_or(false)
    }

    fn unsub(&self, id: &str, path: &P) -> bool {
        self.subs.write().map(|mut coll| {
            let subs = coll.entry(path.as_ref().into()).or_insert(vec![]);
            subs.iter().position(|&(ref t_id, _)| t_id == id).map(|i| subs.remove(i));
            true
        }).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use flag::*;
    use store::*;

    use super::*;

    const PATH: &'static str = "the-owner-uuid-value:app:env";

    fn f<S: Into<String>>(key: S, enabled: bool) -> Flag {
        Flag::new(key, FlagValue::Bool(true), 1, enabled)
    }

    fn path() -> FlagPath {
        PATH.parse::<FlagPath>().unwrap()
    }

    fn dataset(p: &str, dur: u64) -> RedisStore<Flag> {
        let store =
            RedisStore::open("0.0.0.0", 6379, Some(p), Some(Duration::new(dur, 0))).unwrap();
        let flags = vec![f("f1", false), f("f2", true)];

        for flag in flags.into_iter() {
            let _ = store.upsert(&path(), flag.key(), &flag);
        }

        store
    }

    #[test]
    fn test_gets_items() {
        let data = dataset("get_items", 0);

        assert_eq!(data.get(&path(), "f1").unwrap().unwrap(), f("f1", false));
        assert_eq!(data.get(&path(), "f2").unwrap().unwrap(), f("f2", true));
        assert!(data.get(&path(), "f3").unwrap().is_none());
    }

    #[test]
    fn test_gets_all_items() {
        let mut test_map = HashMap::new();
        test_map.insert("f1", f("f1", false));
        test_map.insert("f2", f("f2", true));

        let res = dataset("all_items", 0).get_all(&path());

        assert!(res.is_ok());

        let map = res.unwrap();
        assert_eq!(map.len(), test_map.len());
        assert_eq!(map.get("f1").unwrap(), test_map.get("f1").unwrap());
        assert_eq!(map.get("f2").unwrap(), test_map.get("f2").unwrap());
    }

    #[test]
    fn test_deletes_without_cache() {
        let data = dataset("delete_no_cache", 0);

        assert_eq!(data.get_all(&path()).unwrap().len(), 2);

        // Test flag #1
        let f1 = data.delete(&path(), "f1");
        assert_eq!(f1.unwrap().unwrap(), f("f1", false));

        let f1_2 = data.get(&path(), "f1");
        assert!(f1_2.unwrap().is_none());

        // Test flag #2
        let f2 = data.delete(&path(), "f2");
        assert_eq!(f2.unwrap().unwrap(), f("f2", true));

        let f2_2 = data.get(&path(), "f2");
        assert!(f2_2.unwrap().is_none());

        assert_eq!(data.get_all(&path()).unwrap().len(), 0);
    }

    #[test]
    fn test_deletes_with_cache() {
        let data = dataset("delete_cache", 30);

        assert_eq!(data.get_all(&path()).unwrap().len(), 2);

        // Test flag #1
        let f1 = data.delete(&path(), "f1");
        assert_eq!(f1.unwrap().unwrap(), f("f1", false));

        let f1_2 = data.get(&path(), "f1");
        assert!(f1_2.unwrap().is_none());

        // Test flag #2
        let f2 = data.delete(&path(), "f2");
        assert_eq!(f2.unwrap().unwrap(), f("f2", true));

        let f2_2 = data.get(&path(), "f2");
        assert!(f2_2.unwrap().is_none());

        assert_eq!(data.get_all(&path()).unwrap().len(), 0);
    }

    #[test]
    fn test_delete_changes_timestamp() {
        let data: Box<Store<FlagPath, Flag, Error = BannerError>> = Box::new(dataset("replace_no_cache", 0));
        let _ = data.upsert(&path(), "f1", &f("f1", true));
        let t1 = data.updated_at().unwrap();
        ::std::thread::sleep(::std::time::Duration::from_millis(50));
        let _ = data.delete(&path(), "f1");
        let t2 = data.updated_at().unwrap();

        assert!(t2 > t1);
    }

    #[test]
    fn test_replacements_without_cache() {
        let data = dataset("replace_no_cache", 0);

        assert_eq!(data.get_all(&path()).unwrap().len(), 2);

        // Test flag #1
        let f1 = data.upsert(&path(), "f1", &f("f1", true));
        assert_eq!(f1.unwrap().unwrap(), f("f1", false));

        let f1_2 = data.get(&path(), "f1");
        assert_eq!(f1_2.unwrap().unwrap(), f("f1", true));

        assert_eq!(data.get_all(&path()).unwrap().len(), 2);
    }

    #[test]
    fn test_replacements_with_cache() {
        let data = dataset("replace_cache", 30);

        assert_eq!(data.get_all(&path()).unwrap().len(), 2);

        // Test flag #1
        let f1 = data.upsert(&path(), "f1", &f("f1", true));
        assert_eq!(f1.unwrap().unwrap(), f("f1", false));

        let f1_2 = data.get(&path(), "f1");
        assert_eq!(f1_2.unwrap().unwrap(), f("f1", true));

        assert_eq!(data.get_all(&path()).unwrap().len(), 2);
    }

    #[test]
    fn test_update_changes_timestamp() {
        let data: Box<Store<FlagPath, Flag, Error = BannerError>> = Box::new(dataset("replace_no_cache", 0));
        let t1 = data.updated_at().unwrap();
        ::std::thread::sleep(::std::time::Duration::from_millis(50));
        let _ = data.upsert(&path(), "f1", &f("f1", true));
        let t2 = data.updated_at().unwrap();

        assert!(t2 > t1);
    }

    #[test]
    fn test_adds_subs() {
        let data = dataset("replace_no_cache", 0);
        data.sub("test-uid", &path(), None);

        let mut m = HashMap::new();
        m.insert(PATH.to_string(), 1);
        
        assert_eq!(data.subs(), m);
    }

    #[test]
    fn test_removes_subs() {
        let data = dataset("replace_no_cache", 0);
        data.sub("test-uid", &path(), None);
        data.unsub("test-uid", &path());

        let mut m = HashMap::new();
        m.insert(PATH.to_string(), 0);
        
        assert_eq!(data.subs(), m);
    }
}
