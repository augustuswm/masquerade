use redis_async::error::Error as RedisAsyncError;
use serde_json::Error as SerdeError;

use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    CachePoisonedError,
    FailedToParsePath,
    RedisAsyncFailure(RedisAsyncError),
    RedisAsyncSubMessageFailure,
    InvalidRedisConfig,
    AllCacheMissing,
    FailedToSerializeItem,
    UpdatedAtPoisoned,
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
            Error::CachePoisonedError => "Failed to access cache due to poisoning",
            Error::FailedToParsePath => "Unable to parse into path",
            Error::RedisAsyncFailure(err) => err.description(),
            Error::RedisAsyncSubMessageFailure => "Async Redis message failed",
            Error::InvalidRedisConfig => "Can not create RedisStore from invalid config",
            Error::AllCacheMissing => "Full cache is misconfigured",
            Error::FailedToSerializeItem => "Failed to turn item into json",
            Error::UpdatedAtPoisoned => "",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}
