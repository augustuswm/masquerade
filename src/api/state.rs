use backend_async::AsyncRedisStore;
use flag::{Flag, FlagPath};
use user::User;

pub type AsyncFlagStore = AsyncRedisStore<FlagPath, Flag>;
pub type AsyncFlagPathStore = AsyncRedisStore<String, FlagPath>;
pub type AsyncUserStore = AsyncRedisStore<String, User>;

pub struct AppState {
    flag_store: AsyncFlagStore,
    path_store: AsyncFlagPathStore,
    user_store: AsyncUserStore
}

impl AppState {
    pub fn new(
        flag_store: AsyncFlagStore,
        path_store: AsyncFlagPathStore,
        user_store: AsyncUserStore) -> AppState
    {
        AppState {
            flag_store: flag_store,
            path_store: path_store,
            user_store: user_store
        }
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
