use std::ops::Deref;

use error::BannerError;
use flag::{Flag, FlagPath};
use store::ThreadedStore;

pub struct AppState {
    flag_store: Box<ThreadedStore<FlagPath, Flag, Error = BannerError>>,
    path_store: Box<ThreadedStore<String, FlagPath, Error = BannerError>>,
}

impl AppState {
    pub fn new<F, P>(flag_store: F, path_store: P) -> AppState
    where
        F: ThreadedStore<FlagPath, Flag, Error = BannerError> + 'static,
        P: ThreadedStore<String, FlagPath, Error = BannerError> + 'static,
    {
        AppState {
            flag_store: Box::new(flag_store),
            path_store: Box::new(path_store),
        }
    }

    pub fn flags(&self) -> &Box<ThreadedStore<FlagPath, Flag, Error = BannerError>> {
        &self.flag_store
    }

    pub fn paths(&self) -> &Box<ThreadedStore<String, FlagPath, Error = BannerError>> {
        &self.path_store
    }
}
