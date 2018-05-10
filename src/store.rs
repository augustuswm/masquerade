use futures::task::Task;

use std::collections::HashMap;
use std::time::Instant;

pub trait Store<Path, Item> {
    type Error;

    fn get(&self, path: &Path, key: &str) -> Result<Option<Item>, Self::Error>;
    fn get_all(&self, path: &Path) -> Result<HashMap<String, Item>, Self::Error>;
    fn delete(&self, path: &Path, key: &str) -> Result<Option<Item>, Self::Error>;
    fn upsert(&self, path: &Path, key: &str, item: &Item) -> Result<Option<Item>, Self::Error>;
    fn updated_at(&self) -> Result<Instant, Self::Error>;
    fn sub(&self, id: &str, path: &Path, task: Task) -> bool;
    fn unsub(&self, id: &str, path: &Path);
}

pub trait ThreadedStore<P, I>: Store<P, I> + Send + Sync {}
impl<T, P, I> ThreadedStore<P, I> for T
where
    T: Store<P, I> + Send + Sync,
{
}
