use iron::{typemap, BeforeMiddleware};
use iron::prelude::*;

use std::sync::Arc;

use error::BannerError;
use flag::Flag;
use store::ThreadedStore;

#[derive(Debug)]
pub struct BackendMiddleware {
    store: Arc<ThreadedStore<Item = Flag, Error = BannerError>>,
}

impl BackendMiddleware {
    pub fn new<T>(store: T) -> BackendMiddleware
    where
        T: ThreadedStore<Item = Flag, Error = BannerError> + 'static,
    {
        BackendMiddleware {
            store: Arc::new(store),
        }
    }
}

impl typemap::Key for BackendMiddleware {
    type Value = Arc<ThreadedStore<Item = Flag, Error = BannerError>>;
}

impl BeforeMiddleware for BackendMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions
            .insert::<BackendMiddleware>(self.store.clone());
        Ok(())
    }
}

pub trait BackendReqExt {
    fn get_store(&self) -> Option<Arc<ThreadedStore<Item = Flag, Error = BannerError>>>;
}

impl<'a, 'b> BackendReqExt for Request<'a, 'b> {
    fn get_store(&self) -> Option<Arc<ThreadedStore<Item = Flag, Error = BannerError>>> {
        self.extensions
            .get::<BackendMiddleware>()
            .map(|backend| backend.clone())
    }
}
