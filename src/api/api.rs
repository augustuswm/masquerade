use actix_web::{Application, Method};

use api::State;
use api::flag;
use api::path;

pub fn app(state: State) -> Application<State> {
    Application::with_state(state)
        .prefix("/api/v1")
        .resource("/{app}/{env}/flag/", |r| {
            r.method(Method::POST).a(flag::create)
        })
        .resource("/{app}/{env}/flag/{key}/", |r| {
            r.method(Method::GET).a(flag::read)
        })
        .resource("/{app}/{env}/flag/{key}/", |r| {
            r.method(Method::POST).a(flag::update)
        })
        .resource("/{app}/{env}/flag/{key}/", |r| {
            r.method(Method::DELETE).a(flag::delete)
        })
        .resource("/{app}/{env}/flags/", |r| {
            r.method(Method::GET).a(flag::all)
        })
        .resource("/path/", |r| r.method(Method::POST).a(path::create))
        .resource("/paths/", |r| r.method(Method::GET).a(path::all))
}
