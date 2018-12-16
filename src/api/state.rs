use std::sync::Arc;

use backend_async::AsyncRedisStore;
use flag::{Flag, FlagPath};
use user::User;

pub type AsyncFlagStore = AsyncRedisStore<FlagPath, Flag>;
pub type AsyncFlagPathStore = AsyncRedisStore<String, FlagPath>;
pub type AsyncUserStore = AsyncRedisStore<String, User>;

pub type Salt = Arc<[u8;16]>;

pub struct Config {
    pub salt: Salt
}

impl Config {
    pub fn new<S>(salt: S) -> Config where S: Into<[u8;16]> {
        Config {
            salt: Arc::new(salt.into())
        }
    }
}

pub struct AppState {
    flag_store: AsyncFlagStore,
    path_store: AsyncFlagPathStore,
    user_store: AsyncUserStore,
    config: Config
}

impl AppState {
    pub fn new(
        flag_store: AsyncFlagStore,
        path_store: AsyncFlagPathStore,
        user_store: AsyncUserStore) -> AppState
    {
        let mut salt: [u8; 16] = [0; 16];
        salt.copy_from_slice(::std::env::var("SALT").unwrap().as_bytes());

        AppState {
            flag_store: flag_store,
            path_store: path_store,
            user_store: user_store,
            config: Config::new(salt)
        }
    }

    pub fn salt(&self) -> &Salt {
        &self.config.salt
    }

    pub fn flags(&self) -> &AsyncFlagStore {
        &self.flag_store
    }

    pub fn paths(&self) -> &AsyncFlagPathStore {
        &self.path_store
    }

    pub fn users(&self) -> &AsyncUserStore {
        &self.user_store
    }
}
