use std::collections::HashMap;

pub trait Store<Path, Item> {
    type Error;

    // fn index(&self) -> Result<Vec<Path>, Self::Error>;
    fn get(&self, path: &Path, key: &str) -> Result<Option<Item>, Self::Error>;
    fn get_all(&self, path: &Path) -> Result<HashMap<String, Item>, Self::Error>;
    fn delete(&self, path: &Path, key: &str) -> Result<Option<Item>, Self::Error>;
    fn upsert(&self, path: &Path, key: &str, item: &Item) -> Result<Option<Item>, Self::Error>;
}

pub trait ThreadedStore<P, I>: Store<P, I> + Send + Sync {}
impl<T, P, I> ThreadedStore<P, I> for T
where
    T: Store<P, I> + Send + Sync,
{
}
