use actix_web::*;

use std::sync::Arc;

use error::BannerError;
use flag::{Flag, FlagPath};
use store::ThreadedStore;

mod api;
mod error;
mod flag;
mod frontend;
mod state;

fn index(req: HttpRequest) -> &'static str {
    "Hello world!"
}

type State = Arc<state::AppState>;

pub fn boot<T>(flags: T)
where
    T: ThreadedStore<FlagPath, Flag, Error = BannerError> + 'static,
{
    let state = Arc::new(state::AppState::new(flags));
    // HttpServer::new(|| Application::new().resource("/", |r| r.f(index)))
    //     .bind("127.0.0.1:8088")
    //     .expect("Can not bind to 127.0.0.1:8088")
    //     .run();
    HttpServer::new(move || vec![api::app(state.clone())])
        .bind("127.0.0.1:8088")
        .expect("Can not bind to 127.0.0.1:8088")
        .run();
}
