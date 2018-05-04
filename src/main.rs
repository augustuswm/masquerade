// #![allow(dead_code)]

extern crate actix;
extern crate actix_web;
extern crate base64;
#[macro_use]
#[cfg(feature = "mongo-backend")]
extern crate bson;
extern crate env_logger;
extern crate futures;
extern crate http;
#[cfg(feature = "dynamo-backend")]
extern crate hyper;
#[macro_use]
extern crate log;
#[cfg(feature = "mongo-backend")]
extern crate mongo_driver;
#[cfg(feature = "redis-backend")]
extern crate redis;
extern crate ring;
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

use store::Store;

mod api;
mod error;
mod flag;
mod hash_cache;
mod storage;
mod store;
mod user;

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    #[cfg(not(any(feature = "dynamo-backend", feature = "redis-backend",
                  feature = "mem-backend", feature = "mongo-backend")))]
    compile_error!("At least one backend feature must be selected");

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

    #[cfg(feature = "dynamo-backend")]
    let users = storage::dynamo::DynamoStore::new("users").unwrap();

    #[cfg(feature = "mem-backend")]
    let users = storage::mem::MemStore::new();

    #[cfg(feature = "mongo-backend")]
    let users = storage::mongo::MongoStore::open("0.0.0.0", 27017, "banner", "", "", None).unwrap();

    #[cfg(feature = "redis-backend")]
    let users = storage::redis::RedisStore::open("0.0.0.0", 6379, Some("banner"), None).unwrap();

    let flag = flag::Flag::new("f1", flag::FlagValue::Bool(true), 1, true);

    let u = user::User::new(
        "user-id".to_string(),
        "dev".to_string(),
        "dev".to_string(),
        true,
    );

    let a = (u.uuid.clone() + ":tpt:prod")
        .parse::<flag::FlagPath>()
        .unwrap();
    // let b = "tpt:staging".parse::<flag::FlagPath>().unwrap();
    // let c = "nextavenue:prod".parse::<flag::FlagPath>().unwrap();
    // let d = "nextavenue:staging".parse::<flag::FlagPath>().unwrap();
    // let e = "rewire:prod".parse::<flag::FlagPath>().unwrap();
    // let f = "rewire:staging".parse::<flag::FlagPath>().unwrap();
    // let g = "id:prod".parse::<flag::FlagPath>().unwrap();
    // let h = "id:staging".parse::<flag::FlagPath>().unwrap();
    // let i = "mm-api:prod".parse::<flag::FlagPath>().unwrap();
    // let j = "mm-api:staging".parse::<flag::FlagPath>().unwrap();

    let _ = apps.upsert(&"paths".to_string(), &(u.uuid.clone() + ":tpt:prod"), &a);
    // let _ = apps.upsert(&"paths".to_string(), "tpt:staging", &b);
    // let _ = apps.upsert(&"paths".to_string(), "nextavenue:prod", &c);
    // let _ = apps.upsert(&"paths".to_string(), "nextavenue:staging", &d);
    // let _ = apps.upsert(&"paths".to_string(), "rewire:prod", &e);
    // let _ = apps.upsert(&"paths".to_string(), "rewire:staging", &f);
    // let _ = apps.upsert(&"paths".to_string(), "id:prod", &g);
    // let _ = apps.upsert(&"paths".to_string(), "id:staging", &h);
    // let _ = apps.upsert(&"paths".to_string(), "mm-api:prod", &i);
    // let _ = apps.upsert(&"paths".to_string(), "mm-api:staging", &j);

    let _ = flags.upsert(&a, "f1", &flag);
    let _ = users.upsert(&"users".to_string(), "dev", &u);

    api::boot(flags, apps, users);

    // let mut entry = Mount::new();

    // entry.mount("/", api::frontend::v1());

    // entry.mount("/api/v1/", api::api::v1(apps, flags));

    // println!("Starting up");
    // Iron::new(entry).http("0.0.0.0:3000").unwrap();
}
