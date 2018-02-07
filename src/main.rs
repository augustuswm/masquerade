#![allow(dead_code, unused_must_use, unused_variables)]

extern crate iron;
#[macro_use]
extern crate log;
extern crate mount;
#[cfg(feature = "redis-backend")]
extern crate redis;
extern crate router;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use iron::Iron;
use mount::Mount;

mod api;
mod app;
mod env;
mod error;
mod flag;
mod hash_cache;
mod storage;
mod store;

fn main() {
    let store = storage::mem::MemStore::new();

    let mut entry = Mount::new();
    entry.mount("/api/v1/", api::v1(store));

    Iron::new(entry).http("localhost:3000").unwrap();
}
