use base64::encode;
use config::{Config as ConfigBuilder, File, Environment};
use log::Level;
use ring::rand::{SecureRandom, SystemRandom};
use serde_derive::Deserialize;

use std::net::SocketAddr;

use crate::api::config::APIConfig;
use crate::error::Error;

#[derive(Debug, Deserialize)]
pub struct CacheConfig {
  duration: u16
}

#[derive(Debug, Deserialize)]
pub struct DBConfig {
  redis: SocketAddr,
  prefix: String,
  cache: CacheConfig
}

#[derive(Debug, Deserialize)]
pub struct Config {

  // FIXME: Looks to be a bug in config-rs that Level enum does not
  // deserialize properly. Wrapping in a struct and flattening
  // seems to work. https://github.com/mehcode/config-rs/issues/74
  #[serde(flatten)]
  log: LogWrapper,
  database: DBConfig,
  api: APIConfig,
}

#[derive(Debug, Deserialize)]
struct LogWrapper {
  log: Level
}

impl Config {
  pub fn new() -> Result<Config, Error> {
    let mut builder = ConfigBuilder::new();

    builder.merge(File::with_name("config")).map_err(Error::InvalidConfig)?;

    builder.merge(Environment::with_prefix("MASQUERADE").separator("_")).map_err(Error::InvalidConfig)?;

    builder.try_into().map_err(Error::InvalidConfig)
  }

  pub fn log_level(&self) -> Level {
    self.log.log
  }

  pub fn db(&self) -> SocketAddr {
    self.database.redis
  }

  pub fn prefix(&self) -> &str {
    &self.database.prefix
  }

  pub fn cache_duration(&self) -> u16 {
    self.database.cache.duration
  }

  pub fn api(&self) -> APIConfig {
    self.api.clone()
  }
}

pub fn generate_secret() -> String {
  let mut dest: [u8; 16] = [0; 16];

  match SystemRandom::new().fill(&mut dest) {
    Ok(_) => encode(&dest),
    Err(err) => {
      println!("Failed to generate random data: {}", err);
      "".to_string()
    },
  }
}
