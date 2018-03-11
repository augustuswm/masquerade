use std::ops::Deref;

use error::BannerError;
use flag::{Flag, FlagPath};
use store::ThreadedStore;

pub struct AppState {
    flag_store: Box<ThreadedStore<FlagPath, Flag, Error = BannerError>>,
}

impl AppState {
    pub fn new<F>(flag_store: F) -> AppState
    where
        F: ThreadedStore<FlagPath, Flag, Error = BannerError> + 'static,
    {
        AppState {
            flag_store: Box::new(flag_store),
        }
    }

    pub fn flags(&self) -> &Box<ThreadedStore<FlagPath, Flag, Error = BannerError>> {
        &self.flag_store
    }
}
