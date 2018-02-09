#![allow(dead_code, unused_must_use, unused_variables)]

extern crate bodyparser;
extern crate iron;
#[macro_use]
extern crate log;
extern crate mount;
extern crate persistent;
#[cfg(feature = "redis-backend")]
extern crate redis;
extern crate router;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate staticfile;

use iron::Iron;
use mount::Mount;
use staticfile::Static;

use std::path::Path;

use store::Store;

mod api;
mod error;
mod flag;
mod hash_cache;
mod storage;
mod store;

fn main() {
    #[cfg(feature = "mem-backend")]
    let backend = storage::mem::MemStore::new();

    #[cfg(feature = "redis-backend")]
    let backend = storage::redis::RedisStore::open("0.0.0.0", 6379, Some("banner"), None).unwrap();

    let flag = flag::Flag::new("f1", "app", "env", flag::FlagValue::Bool(true), 1, true);

    backend.upsert("tpt::prod", "f1", &flag);

    let mut entry = Mount::new();

    entry.mount("/", Static::new(Path::new("src/frontend/static/")));

    entry.mount("/api/v1/", api::v1(backend));

    Iron::new(entry).http("localhost:3000").unwrap();
}
