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
use uuid::Uuid;

use std::env;
use std::net::{SocketAddr, ToSocketAddrs};

mod api;
#[macro_use]
mod backend_async;
mod error;
mod flag;
mod hash_cache;
mod user;

const DEFAULT_USER: &'static str = "dev";
const DEFAULT_PASS: &'static str = "dev";

fn run<F>(to_run: F) -> F::Item where F: Future {
    Runtime::new().unwrap().block_on(to_run).map_err(|_| ()).unwrap()
}

fn launch(host: &str, port: &str, prefix: &str) -> Result<(), &'static str> {
    let addresses: Vec<SocketAddr> = (host.to_string() + ":" + port).to_socket_addrs().map_err(|_| "Failed to parse address for Redis")?.collect();

    if addresses.len() == 0 {
        return Err("Failed to resolve Redis host");
    }

    let address = addresses[0];

    let flags = backend_async::AsyncRedisStore::open(
        address,
        prefix,
        Some(prefix),
        None,
    );

    let apps = backend_async::AsyncRedisStore::open(
        address,
        prefix,
        Some(prefix),
        None,
    );

    let users = backend_async::AsyncRedisStore::open(
        address,
        prefix,
        Some(prefix),
        None,
    );

    let user = user::User::new(
        Uuid::new_v4().to_string(),
        DEFAULT_USER.to_string(),
        DEFAULT_PASS.to_string(),
        true,
    );

    let _ = run(users.upsert(&"users".to_string(), "dev", &user));

    api::boot(flags, apps, users);

    Ok(())
}

fn main() -> Result<(), &'static str> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    launch(
        &env::var("REDIS_HOST").unwrap_or("127.0.0.1".to_string()),
        &env::var("REDIS_PORT").unwrap_or("6379".to_string()),
        &env::var("REDIS_PREFIX").unwrap_or("masquerade".to_string()),

    )
}
