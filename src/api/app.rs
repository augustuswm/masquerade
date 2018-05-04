use actix_web::{fs, Application, Method};
use actix_web::middleware::Logger;

use api::auth;
use api::flag;
use api::path;
use api::State;

pub fn api(state: State) -> Application<State> {
    Application::with_state(state)
        .prefix("/api/v1")
        .middleware(Logger::default())
        .middleware(auth::BasicAuth)
        .resource("/{app}/{env}/flag/", |r| {
            r.method(Method::POST).a(flag::create)
        })
        .resource("/{app}/{env}/flag/{key}/", |r| {
            r.method(Method::GET).a(flag::read);
            r.method(Method::POST).a(flag::update);
            r.method(Method::DELETE).a(flag::delete)
        })
        .resource("/{app}/{env}/flags/", |r| {
            r.method(Method::GET).a(flag::all)
        })
        .resource("/path/", |r| r.method(Method::POST).a(path::create))
        .resource("/paths/", |r| r.method(Method::GET).a(path::all))
}

pub fn frontend(state: State) -> Application<State> {
    Application::with_state(state)
        .middleware(Logger::default())
        .handler(
            "/",
            fs::StaticFiles::new("www/", false).index_file("index.html"),
        )
}
