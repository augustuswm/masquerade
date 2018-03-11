use iron::{typemap, BeforeMiddleware};
use iron::prelude::*;

use std::sync::Arc;

use error::BannerError;
use store::ThreadedStore;

pub struct BackendMiddleware<Path, Item> {
    store: Arc<ThreadedStore<Path, Item, Error = BannerError>>,
}

impl<Path, Item> BackendMiddleware<Path, Item> {
    pub fn new<T>(store: T) -> BackendMiddleware<Path, Item>
    where
        T: ThreadedStore<Path, Item, Error = BannerError> + 'static,
    {
        BackendMiddleware {
            store: Arc::new(store),
        }
    }
}

impl<Path: 'static, Item: 'static> typemap::Key for BackendMiddleware<Path, Item> {
    type Value = Arc<ThreadedStore<Path, Item, Error = BannerError>>;
}

impl<Path: 'static, Item: 'static> BeforeMiddleware for BackendMiddleware<Path, Item> {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions
            .insert::<BackendMiddleware<Path, Item>>(self.store.clone());
        Ok(())
    }
}

pub trait BackendReqExt<Path, Item> {
    fn get_store(&self) -> Option<Arc<ThreadedStore<Path, Item, Error = BannerError>>>;
}

impl<'a, 'b, Path: 'static, Item: 'static> BackendReqExt<Path, Item> for Request<'a, 'b> {
    fn get_store(&self) -> Option<Arc<ThreadedStore<Path, Item, Error = BannerError>>> {
        self.extensions
            .get::<BackendMiddleware<Path, Item>>()
            .map(|backend| backend.clone())
    }
}
