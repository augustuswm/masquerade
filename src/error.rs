use config::ConfigError;
use redis_async::error::Error as RedisAsyncError;
use serde_json::Error as SerdeError;

use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    InvalidConfig(ConfigError),
    FailedToParsePath,
    EmptyKey,
    RedisAsyncFailure(RedisAsyncError),
    RedisAsyncSubMessageFailure,
    InvalidRedisConfig,
    AllCacheMissing,
    FailedToSerializeItem,
}

impl From<RedisAsyncError> for Error {
    fn from(err: RedisAsyncError) -> Error {
        Error::RedisAsyncFailure(err)
    }
}

impl From<SerdeError> for Error {
    fn from(_: SerdeError) -> Error {
        Error::FailedToSerializeItem
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::InvalidConfig(err) => err.description(),
            Error::FailedToParsePath => "Unable to parse into path",
            Error::EmptyKey => "Unable to operate on an empty key",
            Error::RedisAsyncFailure(err) => err.description(),
            Error::RedisAsyncSubMessageFailure => "Async Redis message failed",
            Error::InvalidRedisConfig => "Can not create RedisStore from invalid config",
            Error::AllCacheMissing => "Full cache is misconfigured",
            Error::FailedToSerializeItem => "Failed to turn item into json",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}
