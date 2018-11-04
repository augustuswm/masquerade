use actix_web::*;
use tokio::executor::spawn;

use std::sync::{Arc, Once, ONCE_INIT};

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

static SYNC_OBJ: Once = ONCE_INIT;

pub fn init_listener(a_flag_store: &state::AsyncFlagStore) {
  SYNC_OBJ.call_once(|| {
    println!("wire listener");
    spawn(a_flag_store.updater());
  });
}

pub fn boot(flags: state::FlagStore, aflags: state::AsyncFlagStore, paths: state::PathStore, users: state::UserStore)
{
    let state = Arc::new(state::AppState::new(flags, aflags, paths, users));

    server::new(move || {
      init_listener(state.aflags());
      vec![app::api(state.clone()), app::frontend(state.clone())]
    })
        .bind("0.0.0.0:8088")
        .expect("Can not bind to 0.0.0.0:8088")
        .run();
}
