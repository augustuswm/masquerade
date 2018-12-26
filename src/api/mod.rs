use actix_web::*;
use log::debug;
use tokio::executor::spawn;

use std::sync::{Arc, Once, ONCE_INIT};

use crate::api::config::APIConfig;

mod admin;
mod app;
mod auth;
pub mod config;
mod error;
mod flag;
mod flag_req;
mod path;
mod state;
mod stream;
mod user;

type State = Arc<state::AppState>;

static SYNC_OBJ: Once = ONCE_INIT;

pub fn init_listener(a_flag_store: &state::AsyncFlagStore) {
    SYNC_OBJ.call_once(|| {
        spawn(a_flag_store.updater());
    });
}

pub fn boot(
    flags: state::AsyncFlagStore,
    paths: state::AsyncFlagPathStore,
    users: state::AsyncUserStore,
    config: APIConfig,
) {
    debug!("API startup");

    let state = Arc::new(state::AppState::new(flags, paths, users, config));

    server::new(move || {
        init_listener(state.flags());
        vec![app::api(state.clone()), app::frontend(state.clone())]
    })
    .bind("0.0.0.0:8088")
    .expect("Can not bind to 0.0.0.0:8088")
    .run();
}
