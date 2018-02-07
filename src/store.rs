use std::collections::HashMap;
use std::fmt::Debug;

pub trait Store {
    type Item;
    type Error;

    fn get(&self, key: &str) -> Result<Option<Self::Item>, Self::Error>;
    fn get_all(&self) -> Result<HashMap<String, Self::Item>, Self::Error>;
    fn delete(&self, key: &str) -> Result<Option<Self::Item>, Self::Error>;
    fn upsert(&self, key: &str, item: &Self::Item) -> Result<Option<Self::Item>, Self::Error>;
}

pub trait ThreadedStore: Store + Send + Sync + Debug {}
impl<T> ThreadedStore for T
where
    T: Store + Send + Sync + Debug,
{
}
