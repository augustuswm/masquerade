use std::collections::HashMap;
use std::time::Duration;

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

impl<T, P> Store<P, T> for MemStore<T>
where
    P: AsRef<str>,
    T: Clone,
{
    type Error = BannerError;

    fn get(&self, path: &P, key: &str) -> Result<Option<T>, BannerError> {
        self.data.get([path.as_ref(), "$", key].concat().as_str())
    }

    fn get_all(&self, path: &P) -> Result<HashMap<String, T>, BannerError> {
        self.data.get_all().map(|map| {
            let mut ret_map: HashMap<String, T> = HashMap::new();
            let pref: String = [path.as_ref(), "$"].concat();

            for (k, v) in map.iter() {
                if k.starts_with(path.as_ref()) {
                    let new_k: String = k.trim_left_matches(pref.as_str()).to_string();
                    ret_map.insert(new_k, v.clone());
                };
            }

            ret_map
        })
    }

    fn delete(&self, path: &P, key: &str) -> Result<Option<T>, BannerError> {
        self.data
            .remove([path.as_ref(), "$", key].concat().as_str())
    }

    fn upsert(&self, path: &P, key: &str, item: &T) -> Result<Option<T>, BannerError> {
        self.data
            .insert([path.as_ref(), "$", key].concat().as_str(), item)
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

    fn dataset() -> MemStore<Flag> {
        let store = MemStore::new();
        let flags = vec![f("f1", false), f("f2", true)];

        for flag in flags.into_iter() {
            store.upsert(&path(), flag.key(), &flag);
        }

        store
    }

    #[test]
    fn test_gets_items() {
        let data = dataset();

        assert_eq!(data.get(&path(), "f1").unwrap().unwrap(), f("f1", false));
        assert_eq!(data.get(&path(), "f2").unwrap().unwrap(), f("f2", true));
        assert!(data.get(&path(), "f3").unwrap().is_none());
    }

    #[test]
    fn test_gets_all_items() {
        let mut test_map = HashMap::new();
        test_map.insert("f1", f("f1", false));
        test_map.insert("f2", f("f2", true));

        let res = dataset().get_all(&path());

        assert!(res.is_ok());

        let map = res.unwrap();
        assert_eq!(map.len(), test_map.len());
        assert_eq!(map.get("f1").unwrap(), test_map.get("f1").unwrap());
        assert_eq!(map.get("f2").unwrap(), test_map.get("f2").unwrap());
    }

    #[test]
    fn test_deletes() {
        let data = dataset();

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
        let data = dataset();

        assert_eq!(data.get_all(&path()).unwrap().len(), 2);

        // Test flag #1
        let f1 = data.upsert(&path(), "f1", &f("f1", true));
        assert_eq!(f1.unwrap().unwrap(), f("f1", false));

        let f1_2 = data.get(&path(), "f1");
        assert_eq!(f1_2.unwrap().unwrap(), f("f1", true));

        assert_eq!(data.get_all(&path()).unwrap().len(), 2);
    }
}
