use actix_web::{fs, App, HttpRequest, Result};
use actix_web::fs::NamedFile;
use actix_web::http::Method;
use actix_web::middleware::Logger;

use std::path::Path;

use api::auth;
use api::flag;
use api::path;
use api::State;
use api::stream;

fn index<'r>(_req: &'r HttpRequest<State>) -> Result<NamedFile> {
    Ok(NamedFile::open(Path::new("www/index.html"))?)
}

pub fn api(state: State) -> App<State> {
    App::with_state(state)
        .prefix("/api/v1")
        .middleware(Logger::default())
        .middleware(auth::UrlAuth)
        .middleware(auth::BasicAuth)
        .resource("/{app}/{env}/flag/", |r| {
            r.method(Method::POST).a(flag::acreate)
        })
        .resource("/{app}/{env}/flag/{key}/", |r| {
            r.method(Method::GET).a(flag::aread);
            r.method(Method::POST).a(flag::aupdate);
            r.method(Method::DELETE).a(flag::adelete)
        })
        .resource("/{app}/{env}/flags/", |r| {
            r.method(Method::GET).a(flag::aall)
        })
        .resource("/path/", |r| r.method(Method::POST).a(path::create))
        .resource("/paths/", |r| r.method(Method::GET).a(path::all))
        .resource("/stream/{app}/{env}/", |r| r.method(Method::GET).a(stream::flag_stream))
}

pub fn frontend(state: State) -> App<State> {
    App::with_state(state)
        .middleware(Logger::default())
        .resource("/", |r| r.h(index))
        .resource("/{app}/{env}/", |r| r.h(index))
        .handler(
            "/",
            fs::StaticFiles::new("www/").expect("Failed to locate www directory").index_file("index.html"),
        )
}
