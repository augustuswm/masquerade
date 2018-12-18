use crate::api::config::APIConfig;
use crate::backend_async::AsyncRedisStore;
use crate::flag::{Flag, FlagPath};
use crate::user::User;

pub type AsyncFlagStore = AsyncRedisStore<FlagPath, Flag>;
pub type AsyncFlagPathStore = AsyncRedisStore<String, FlagPath>;
pub type AsyncUserStore = AsyncRedisStore<&'static str, User>;

pub struct AppState {
    flag_store: AsyncFlagStore,
    path_store: AsyncFlagPathStore,
    user_store: AsyncUserStore,
    config: APIConfig
}

impl AppState {
    pub fn new(
        flag_store: AsyncFlagStore,
        path_store: AsyncFlagPathStore,
        user_store: AsyncUserStore,
        config: APIConfig) -> AppState
    {
        AppState {
            flag_store: flag_store,
            path_store: path_store,
            user_store: user_store,
            config: config
        }
    }

    pub fn jwt_secret(&self) -> &str {
        &self.config.jwt_secret
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
