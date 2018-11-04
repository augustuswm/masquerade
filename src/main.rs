// #![allow(dead_code)]

extern crate actix;
extern crate actix_web;
extern crate base64;
extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate http;
#[macro_use]
extern crate log;
extern crate redis;
#[macro_use]
extern crate redis_async;
extern crate ring;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio;
extern crate uuid;

use std::env;

mod api;
mod error;
mod flag;
mod hash_cache;
mod backend;
mod backend_async;
mod user;

fn sync() {
    let apps = backend::RedisStore::open(
        env::var("REDIS_HOST").unwrap_or("redis".to_string()),
        6379,
        Some("banner"),
        None,
    ).unwrap();

    let flags = backend::RedisStore::open(
        env::var("REDIS_HOST").unwrap_or("redis".to_string()),
        6379,
        Some("banner"),
        None,
    ).unwrap();

    let aflags = backend_async::AsyncRedisStore::open(
        "127.0.0.1:6379".parse().unwrap(),
        "masquerade",
        Some("banner"),
        None,
    );

    let users = backend::RedisStore::open(
        env::var("REDIS_HOST").unwrap_or("redis".to_string()),
        6379,
        Some("banner"),
        None,
    ).unwrap();

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

    let _ = apps.upsert(
        &"paths".to_string(),
        &(u.uuid.clone() + ":test_app:test_env"),
        &a,
    );

    let _ = flags.upsert(&a, "f1", &flag);
    let _ = users.upsert(&"users".to_string(), "dev", &u);

    api::boot(flags, aflags, apps, users);
}

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    sync()
}
