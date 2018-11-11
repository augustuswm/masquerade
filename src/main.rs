// #![allow(dead_code)]

extern crate actix;
extern crate actix_web;
extern crate base64;
extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate http;
extern crate jsonwebtoken;
#[macro_use]
extern crate log;
#[macro_use]
extern crate redis_async;
extern crate ring;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio;
extern crate uuid;

use futures::Future;
use tokio::runtime::current_thread::Runtime;

use std::env;

mod api;
#[macro_use]
mod backend_async;
mod error;
mod flag;
mod hash_cache;
mod user;


fn run<F>(to_run: F) -> F::Item where F: Future {
    Runtime::new().unwrap().block_on(to_run).map_err(|_| ()).unwrap()
}

fn sync() {
    let a_apps = backend_async::AsyncRedisStore::open(
        "127.0.0.1:6379".parse().unwrap(),
        "masquerade",
        Some("banner"),
        None,
    );

    let a_flags = backend_async::AsyncRedisStore::open(
        "127.0.0.1:6379".parse().unwrap(),
        "masquerade",
        Some("banner"),
        None,
    );

    let a_users = backend_async::AsyncRedisStore::open(
        "127.0.0.1:6379".parse().unwrap(),
        "masquerade",
        Some("banner"),
        None,
    );

    let flag = flag::Flag::new("f1", flag::FlagValue::Bool(true), 1, true);

    let u = user::User::new(
        "user-id".to_string(),
        "dev".to_string(),
        "dev".to_string(),
        true,
    );

    let a = (u.uuid.clone() + ":test_app:test_env")
        .parse::<flag::FlagPath>()
        .unwrap();

    let _ = run(a_apps.upsert(
        &"paths".to_string(),
        &(u.uuid.clone() + ":test_app:test_env"),
        &a,
    ));

    let _ = run(a_flags.upsert(&a, "f1", &flag));
    let _ = run(a_users.upsert(&"users".to_string(), "dev", &u));

    api::boot(a_flags, a_apps, a_users);
}

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    sync()
}
