use actix_web::fs::NamedFile;
use actix_web::http::Method;
use actix_web::middleware::Logger;
use actix_web::{fs, App, HttpRequest, Result};

use std::path::Path;

use crate::api::admin;
use crate::api::auth;
use crate::api::flag;
use crate::api::path;
use crate::api::stream;
use crate::api::user;
use crate::api::State;

fn index<'r>(_req: &'r HttpRequest<State>) -> Result<NamedFile> {
    Ok(NamedFile::open(Path::new("www/index.html"))?)
}

pub fn api(state: State) -> App<State> {
    App::with_state(state)
        .prefix("/api/v1")
        .middleware(Logger::default())
        .resource("/authenticate/", |r| {
            r.middleware(auth::BasicAuth);
            r.middleware(auth::RequireUser);
            r.method(Method::POST).a(auth::authenticate)
        })
        .resource("/path/", |r| {
            r.middleware(auth::JWTAuth);
            r.middleware(auth::RequireUser);
            r.method(Method::POST).a(path::create)
        })
        .resource("/paths/", |r| {
            r.middleware(auth::JWTAuth);
            r.middleware(auth::RequireUser);
            r.method(Method::GET).a(path::all)
        })
        .resource("/stream/{app}/{env}/", |r| {
            r.middleware(auth::JWTAuth);
            r.middleware(auth::RequireUser);
            r.method(Method::GET).a(stream::flag_stream)
        })
        .scope("/users", |scope| {
            scope
                .middleware(auth::JWTAuth)
                .middleware(admin::Admin)
                .resource("/{key}/", |r| {
                    r.method(Method::GET).a(user::read);
                    r.method(Method::POST).a(user::update);
                    r.method(Method::DELETE).a(user::delete);
                })
                .resource("/", |r| {
                    r.method(Method::GET).a(user::all);
                    r.method(Method::POST).a(user::create);
                })
        })
        .scope("/{app}/{env}", |scope| {
            scope
                .middleware(auth::JWTAuth)
                .middleware(auth::RequireUser)
                .resource("/flag/", |r| r.method(Method::POST).a(flag::create))
                .resource("/flag/{key}/", |r| {
                    r.method(Method::GET).a(flag::read);
                    r.method(Method::POST).a(flag::update);
                    r.method(Method::DELETE).a(flag::delete)
                })
                .resource("/flags/", |r| r.method(Method::GET).a(flag::all))
        })
}

pub fn frontend(state: State) -> App<State> {
    App::with_state(state)
        .middleware(Logger::default())
        .resource("/", |r| r.h(index))
        .resource("/{app}/{env}/", |r| r.h(index))
        .handler(
            "/",
            fs::StaticFiles::new("www/")
                .expect("Failed to locate www directory")
                .index_file("index.html"),
        )
}
