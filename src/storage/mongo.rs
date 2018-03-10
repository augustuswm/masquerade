use bson;
use bson::{Bson, DecoderError, EncoderError};
use mongo_driver::MongoError as MongoDriverError;
use mongo_driver::client::{ClientPool, Uri};
use mongo_driver::collection::UpdateOptions;
use mongo_driver::flags::UpdateFlag;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

use error::BannerError;
use hash_cache::HashCache;
use store::Store;

#[derive(Debug)]
pub struct MongoStore<T> {
    db: String,
    pool: ClientPool,
    cache: HashCache<T>,
    all_cache: HashCache<HashMap<String, T>>,
    timeout: Duration,
}

pub type MongoStoreResult<T> = Result<T, MongoError>;

#[derive(Debug)]
pub enum MongoError {
    Encode(EncoderError),
    Decode(DecoderError),
    Driver(MongoDriverError),
    IndexNotFound,
    InvalidMongoConfig,
}

impl<T: Clone> MongoStore<T> {
    pub fn open<S, U, V, X>(
        host: S,
        port: u16,
        db_name: U,
        user: V,
        pass: X,
        timeout: Option<Duration>,
    ) -> MongoStoreResult<MongoStore<T>>
    where
        S: Into<String>,
        U: Into<String>,
        V: Into<String>,
        X: Into<String>,
    {
        let user_string = user.into();
        let pass_string = pass.into();

        let url = if user_string != "" && pass_string != "" {
            format!(
                "mongodb://{}:{}@{}:{}",
                user_string,
                pass_string,
                host.into(),
                port
            )
        } else {
            format!("mongodb://{}:{}", host.into(), port)
        };

        MongoStore::open_with_url(url, db_name, timeout)
    }

    pub fn open_with_url<S, U>(
        url: S,
        db_name: U,
        timeout: Option<Duration>,
    ) -> MongoStoreResult<MongoStore<T>>
    where
        S: Into<String>,
        U: Into<String>,
    {
        let uri = Uri::new(url.into()).ok_or(MongoError::InvalidMongoConfig)?;
        let pool = ClientPool::new(uri, None);

        MongoStore::open_with_pool(pool, db_name, timeout)
    }

    pub fn open_with_pool<S>(
        pool: ClientPool,
        db_name: S,
        timeout: Option<Duration>,
    ) -> MongoStoreResult<MongoStore<T>>
    where
        S: Into<String>,
    {
        let dur = timeout.unwrap_or(Duration::new(0, 0));

        let store = MongoStore {
            db: db_name.into(),
            pool: pool,
            cache: HashCache::new(dur),
            all_cache: HashCache::new(dur),
            timeout: dur,
        };

        store.create_idx()?;

        Ok(store)
    }

    pub fn create_idx(&self) -> MongoStoreResult<()> {
        let key_idx = doc! {
            "key" => doc! {"key" => 1},
            "name" => "key"
        };
        let path_idx = doc! {
            "key" => doc! {"path" => 1},
            "name" => "path"
        };
        let idx_cmd = doc! {
            "createIndexes" => "banner_items",
            "indexes" => [key_idx, path_idx]
        };

        let idx_writer = self.pool.pop();
        let coll = idx_writer.get_collection(self.db.as_str(), "banner_items");
        let res = coll.command(idx_cmd, None)
            .map_err(MongoError::Driver)?
            .next();

        res.map(|_| ()).ok_or(MongoError::IndexNotFound)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Wrapper<T> {
    key: String,
    path: String,
    data: T,
}

impl<'de, T, P> Store<P, T> for MongoStore<T>
where
    P: AsRef<str> + Serialize + Deserialize<'de> + Debug,
    T: Clone + Serialize + Deserialize<'de> + Debug,
{
    type Error = BannerError;

    fn get(&self, path: &P, key: &str) -> Result<Option<T>, BannerError> {
        let query = doc! {
            "key" => key,
            "path" => path.as_ref()
        };

        let client = self.pool.pop();
        let coll = client.get_collection(self.db.as_str(), "banner_items");

        let mut cursor = coll.find(&query, None).map_err(MongoError::Driver)?;

        match cursor.next() {
            Some(res) => {
                let doc = res.map_err(MongoError::Driver)?;
                bson::from_bson::<Wrapper<T>>(Bson::Document(doc))
                    .map(|item| Some(item.data))
                    .map_err(|err| MongoError::Decode(err).into())
            }
            None => Ok(None),
        }
    }

    fn get_all(&self, path: &P) -> Result<HashMap<String, T>, BannerError> {
        let query = doc! {
            "path" => path.as_ref()
        };

        let client = self.pool.pop();
        let coll = client.get_collection(self.db.as_str(), "banner_items");

        let cursor = coll.find(&query, None).map_err(MongoError::Driver)?;

        let mut res = HashMap::new();

        for item in cursor {
            let doc = item.map_err(MongoError::Driver)?;
            let wrapper = bson::from_bson::<Wrapper<T>>(Bson::Document(doc))
                .map_err(|err| MongoError::Decode(err))?;

            res.insert(wrapper.key.to_string(), wrapper.data);
        }

        Ok(res)
    }

    fn delete(&self, path: &P, key: &str) -> Result<Option<T>, BannerError> {
        // Ignores cache lookup

        let filter = doc! {
            "key" => key,
            "path" => path.as_ref()
        };

        let existing = self.get(path, key)?;

        let client = self.pool.pop();
        let coll = client.get_collection(self.db.as_str(), "banner_items");

        coll.remove(&filter, None)
            .map(|_| existing)
            .map_err(|err| MongoError::Driver(err).into())
    }

    fn upsert(&self, path: &P, key: &str, item: &T) -> Result<Option<T>, BannerError> {
        // Ignores cache lookup

        let filter = doc! {
            "key" => key,
            "path" => path.as_ref()
        };

        let existing = self.get(path, key)?;

        let doc = doc! {
            "key" => key,
            "path" => path.as_ref(),
            "data" => bson::to_bson(item).map_err(MongoError::Encode)?
        };

        let client = self.pool.pop();
        let coll = client.get_collection(self.db.as_str(), "banner_items");

        let mut opts = UpdateOptions::default();
        opts.update_flags.add(UpdateFlag::Upsert);

        coll.update(&filter, &doc, Some(&opts))
            .map(|_| existing)
            .map_err(|err| MongoError::Driver(err).into())
    }
}

#[cfg(test)]
mod tests {
    use flag::*;
    use store::*;

    use super::*;

    const PATH: &'static str = "app$env";

    fn f<S: Into<String>>(key: S, enabled: bool) -> Flag {
        Flag::new(key, FlagValue::Bool(true), 1, enabled)
    }

    fn path() -> FlagPath {
        PATH.parse::<FlagPath>().unwrap()
    }

    fn dataset(p: &str, dur: u64) -> MongoStore<Flag> {
        let store =
            MongoStore::open("localhost", 27017, p, "", "", Some(Duration::new(dur, 0))).unwrap();
        let flags = vec![f("f1", false), f("f2", true)];

        for flag in flags.into_iter() {
            match store.upsert(&path(), flag.key(), &flag) {
                Ok(_) => (),
                err => panic!("{:?}", err),
            }
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
}
