use actix_web::*;

use std::sync::Arc;

mod admin;
mod app;
mod auth;
mod error;
mod flag;
mod flag_req;
mod path;
mod state;
mod stream;

type State = Arc<state::AppState>;

pub fn boot(flags: state::FlagStore, aflags: state::AsyncFlagStore, paths: state::PathStore, users: state::UserStore)
{
    let state = Arc::new(state::AppState::new(flags, aflags, paths, users));
    server::new(move || vec![app::api(state.clone()), app::frontend(state.clone())])
        .bind("0.0.0.0:8088")
        .expect("Can not bind to 0.0.0.0:8088")
        .run();
}
