use backend_async::AsyncRedisStore;
use backend::RedisStore;
use flag::{Flag, FlagPath};
use user::User;

pub type FlagStore = RedisStore<FlagPath, Flag>;
pub type AsyncFlagStore = AsyncRedisStore<FlagPath, Flag>;
pub type PathStore = RedisStore<String, FlagPath>;
pub type UserStore = RedisStore<String, User>;

pub struct AppState {
    flag_store: FlagStore,
    a_flag_store: AsyncFlagStore,
    path_store: PathStore,
    user_store: UserStore,
}

impl AppState {
    pub fn new(
        flag_store: FlagStore,
        a_flag_store: AsyncFlagStore,
        path_store: PathStore,
        user_store: UserStore) -> AppState
    {
        AppState {
            flag_store: flag_store,
            a_flag_store: a_flag_store,
            path_store: path_store,
            user_store: user_store,
        }
    }

    pub fn flags(&self) -> &FlagStore {
        &self.flag_store
    }

    pub fn aflags(&self) -> &AsyncFlagStore {
        &self.a_flag_store
    }

    pub fn paths(&self) -> &PathStore {
        &self.path_store
    }

    pub fn users(&self) -> &UserStore {
        &self.user_store
    }
}
