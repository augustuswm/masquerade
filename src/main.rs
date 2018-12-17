// #![allow(dead_code)]

use futures::Future;
use structopt::StructOpt;
use tokio::runtime::current_thread::Runtime;
use uuid::Uuid;

use std::env;
use std::time::{Duration};

mod api;
#[macro_use]
mod backend_async;
mod cli;
mod config;
mod error;
mod flag;
mod hash_cache;
mod user;

use crate::config::Config;

const DEFAULT_USER: &'static str = "dev";
const DEFAULT_PASS: &'static str = "dev";

fn run<F>(to_run: F) -> F::Item where F: Future {
    Runtime::new().unwrap().block_on(to_run).map_err(|_| ()).unwrap()
}

fn get_config() -> Result<Config, String> {
    Config::new().map_err(|err| format!("Invalid config: {:?}", err))
}

fn launch(config: Config) -> Result<(), String> {
    env::set_var("RUST_LOG", config.log_level().to_string());
    env_logger::init();


    let flags = backend_async::AsyncRedisStore::open(
        config.db(),
        config.prefix(),
        Some(config.prefix()),
        Some(Duration::new(config.cache_duration().into(), 0)),
    );

    let apps = backend_async::AsyncRedisStore::open(
        config.db(),
        config.prefix(),
        Some(config.prefix()),
        Some(Duration::new(config.cache_duration().into(), 0)),
    );

    let users = backend_async::AsyncRedisStore::open(
        config.db(),
        config.prefix(),
        Some(config.prefix()),
        Some(Duration::new(config.cache_duration().into(), 0)),
    );

    match run(users.get(&"users".to_string(), "dev")) {
        None => {
            let user = user::User::new(
                Uuid::new_v4().to_string(),
                DEFAULT_USER.to_string(),
                DEFAULT_PASS.to_string(),
                true,
            );

            let _ = run(users.upsert(&"users".to_string(), "dev", &user));
        }
        _ => ()
    };

    api::boot(flags, apps, users, config.api());

    Ok(())
}

fn main() -> Result<(), String> {
    let opt = cli::Options::from_args();

    match opt.cmd {
        Some(cli::Command::GenerateSecret) => {
            println!("{}", config::generate_secret());
            Ok(())
        }
        Some(cli::Command::TestConfig) => {
            let _test = get_config()?;
            println!("Config OK");
            Ok(())
        }
        None => launch(get_config()?)
    }
}
