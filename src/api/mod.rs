use actix_web::*;

use std::sync::Arc;

use error::BannerError;
use flag::{Flag, FlagPath};
use store::ThreadedStore;
use user::User;

mod admin;
// mod api;
mod app;
mod auth;
mod error;
mod flag;
mod flag_req;
// mod frontend;
mod path;
mod state;
mod stream;

type State = Arc<state::AppState>;

pub fn boot<T, S, U>(flags: T, paths: S, users: U)
where
    T: ThreadedStore<FlagPath, Flag, Error = BannerError> + 'static,
    S: ThreadedStore<String, FlagPath, Error = BannerError> + 'static,
    U: ThreadedStore<String, User, Error = BannerError> + 'static,
{
    let state = Arc::new(state::AppState::new(flags, paths, users));
    // HttpServer::new(|| Application::new().resource("/", |r| r.f(index)))
    //     .bind("127.0.0.1:8088")
    //     .expect("Can not bind to 127.0.0.1:8088")
    //     .run();
    server::new(move || vec![app::api(state.clone()), app::frontend(state.clone())])
        .bind("0.0.0.0:8088")
        .expect("Can not bind to 0.0.0.0:8088")
        .run();
}
