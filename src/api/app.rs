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
            r.method(Method::POST).with_async(auth::authenticate)
        })
        .resource("/path/", |r| {
            r.middleware(auth::JWTAuth);
            r.middleware(auth::RequireUser);
            r.method(Method::POST).with_async(path::create)
        })
        .resource("/paths/", |r| {
            r.middleware(auth::JWTAuth);
            r.middleware(auth::RequireUser);
            r.method(Method::GET).with_async(path::all)
        })
        .resource("/stream/{app}/{env}/", |r| {
            r.middleware(auth::JWTAuth);
            r.middleware(auth::RequireUser);
            r.method(Method::GET).with_async(stream::flag_stream)
        })
        .scope("/users", |scope| {
            scope
                .middleware(auth::BasicAuth)
                .middleware(admin::Admin)
                .resource("/{key}/", |r| {
                    r.method(Method::GET).with_async(user::read);
                    r.method(Method::POST).with_async(user::update);
                    r.method(Method::DELETE).with_async(user::delete);
                })
                .resource("/", |r| {
                    r.method(Method::GET).with_async(user::all);
                    r.method(Method::POST).with_async(user::create);
                })
        })
        .scope("/{app}/{env}", |scope| {
            scope
                .middleware(auth::JWTAuth)
                .middleware(auth::RequireUser)
                .resource("/flag/", |r| {
                    r.method(Method::POST).with_async(flag::create)
                })
                .resource("/flag/{key}/", |r| {
                    r.method(Method::GET).with_async(flag::read);
                    r.method(Method::POST).with_async(flag::update);
                    r.method(Method::DELETE).with_async(flag::delete)
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
