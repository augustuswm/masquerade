extern crate actix;
extern crate actix_web;
// extern crate bodyparser;
#[macro_use]
#[cfg(feature = "mongo-backend")]
extern crate bson;
// extern crate env_logger;
extern crate futures;
#[cfg(feature = "dynamo-backend")]
extern crate hyper;
// extern crate iron;
#[macro_use]
extern crate log;
// extern crate logger;
#[cfg(feature = "mongo-backend")]
extern crate mongo_driver;
// extern crate mount;
// extern crate persistent;
#[cfg(feature = "redis-backend")]
extern crate redis;
// extern crate router;
#[cfg(feature = "dynamo-backend")]
extern crate rusoto_core;
#[cfg(feature = "dynamo-backend")]
extern crate rusoto_credential;
#[cfg(feature = "dynamo-backend")]
extern crate rusoto_dynamodb;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
// extern crate staticfile;

// use iron::Iron;
// use mount::Mount;

use store::Store;

mod api;
mod error;
mod flag;
mod hash_cache;
mod storage;
mod store;

fn main() {
    #[cfg(not(any(feature = "dynamo-backend", feature = "redis-backend",
                  feature = "mem-backend", feature = "mongo-backend")))]
    compile_error!("At least one backend feature must be selected");

    // env_logger::init();

    #[cfg(feature = "dynamo-backend")]
    let apps = storage::dynamo::DynamoStore::new("apps").unwrap();

    #[cfg(feature = "mem-backend")]
    let apps = storage::mem::MemStore::new();

    #[cfg(feature = "mongo-backend")]
    let apps = storage::mongo::MongoStore::open("0.0.0.0", 27017, "banner", "", "", None).unwrap();

    #[cfg(feature = "redis-backend")]
    let apps = storage::redis::RedisStore::open("0.0.0.0", 6379, Some("banner"), None).unwrap();

    #[cfg(feature = "dynamo-backend")]
    let flags = storage::dynamo::DynamoStore::new("flags").unwrap();

    #[cfg(feature = "mem-backend")]
    let flags = storage::mem::MemStore::new();

    #[cfg(feature = "mongo-backend")]
    let flags = storage::mongo::MongoStore::open("0.0.0.0", 27017, "banner", "", "", None).unwrap();

    #[cfg(feature = "redis-backend")]
    let flags = storage::redis::RedisStore::open("0.0.0.0", 6379, Some("banner"), None).unwrap();

    let flag = flag::Flag::new("f1", flag::FlagValue::Bool(true), 1, true);

    let a = "tpt$prod".parse::<flag::FlagPath>().unwrap();

    let _ = apps.upsert(&"paths".to_string(), "tpt$prod", &a);
    let _ = flags.upsert(&a, "f1", &flag);

    api::boot(flags);

    // let mut entry = Mount::new();

    // entry.mount("/", api::frontend::v1());

    // entry.mount("/api/v1/", api::api::v1(apps, flags));

    // println!("Starting up");
    // Iron::new(entry).http("0.0.0.0:3000").unwrap();
}
