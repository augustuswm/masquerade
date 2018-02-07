use std::collections::HashMap;
use std::time::{Duration, Instant};

use error::BannerError;
use hash_cache::HashCache;
use store::Store;

#[derive(Debug, Clone)]
pub struct MemStore<T> {
    data: HashCache<T>,
}

impl<T> MemStore<T> {
    pub fn new() -> MemStore<T> {
        MemStore {
            data: HashCache::new(Duration::new(0, 0)),
        }
    }
}

impl<T> From<HashMap<String, (T, Instant)>> for MemStore<T> {
    fn from(map: HashMap<String, (T, Instant)>) -> MemStore<T> {
        MemStore { data: map.into() }
    }
}

impl<T: Clone> Store for MemStore<T> {
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

#[cfg(test)]
mod tests {
    use flag::*;
    use store::*;

    use super::*;

    fn f<S: Into<String>>(key: S, enabled: bool) -> Flag {
        Flag::new(key, "", "", FlagValue::Bool(true), 1, enabled)
    }

    fn dataset() -> MemStore<Flag> {
        let store = MemStore::new();
        let flags = vec![f("f1", false), f("f2", true)];

        for flag in flags.into_iter() {
            store.upsert(flag.key(), &flag);
        }

        store
    }

    #[test]
    fn test_gets_items() {
        let data = dataset();

        assert_eq!(data.get("f1").unwrap().unwrap(), f("f1", false));
        assert_eq!(data.get("f2").unwrap().unwrap(), f("f2", true));
        assert!(data.get("f3").unwrap().is_none());
    }

    #[test]
    fn test_gets_all_items() {
        let mut test_map = HashMap::new();
        test_map.insert("f1", f("f1", false));
        test_map.insert("f2", f("f2", true));

        let res = dataset().get_all();

        assert!(res.is_ok());

        let map = res.unwrap();
        assert_eq!(map.len(), test_map.len());
        assert_eq!(map.get("f1").unwrap(), test_map.get("f1").unwrap());
        assert_eq!(map.get("f2").unwrap(), test_map.get("f2").unwrap());
    }

    #[test]
    fn test_deletes() {
        let data = dataset();

        assert_eq!(data.get_all().unwrap().len(), 2);

        // Test flag #1
        let f1 = data.delete("f1");
        assert_eq!(f1.unwrap().unwrap(), f("f1", false));

        let f1_2 = data.get("f1");
        assert!(f1_2.unwrap().is_none());

        // Test flag #2
        let f2 = data.delete("f2");
        assert_eq!(f2.unwrap().unwrap(), f("f2", true));

        let f2_2 = data.get("f2");
        assert!(f2_2.unwrap().is_none());

        assert_eq!(data.get_all().unwrap().len(), 0);
    }

    #[test]
    fn test_replacements_without_cache() {
        let data = dataset();

        assert_eq!(data.get_all().unwrap().len(), 2);

        // Test flag #1
        let f1 = data.upsert("f1", &f("f1", true));
        assert_eq!(f1.unwrap().unwrap(), f("f1", false));

        let f1_2 = data.get("f1");
        assert_eq!(f1_2.unwrap().unwrap(), f("f1", true));

        assert_eq!(data.get_all().unwrap().len(), 2);
    }
}
