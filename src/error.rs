use redis::RedisError;
use redis_async::error::Error as RedisAsyncError;
use serde_json::Error as SerdeError;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum BannerError {
    CachePoisonedError,
    FailedToParsePath,
    RedisFailure(RedisError),
    RedisAsyncFailure(RedisAsyncError),
    RedisAsyncSubMessageFailure,

    InvalidRedisConfig,
    AllCacheMissing,
    FailedToSerializeItem,
    UpdatedAtPoisoned,
}

impl From<RedisError> for BannerError {
    fn from(err: RedisError) -> BannerError {
        BannerError::RedisFailure(err)
    }
}

impl From<RedisAsyncError> for BannerError {
    fn from(err: RedisAsyncError) -> BannerError {
        BannerError::RedisAsyncFailure(err)
    }
}

impl From<SerdeError> for BannerError {
    fn from(_: SerdeError) -> BannerError {
        BannerError::FailedToSerializeItem
    }
}

impl Error for BannerError {
    fn description(&self) -> &str {
        match self {
            BannerError::CachePoisonedError => "Failed to access cache due to poisoning",
            BannerError::FailedToParsePath => "Unable to parse into path",
            BannerError::RedisFailure(err) => err.description(),
            BannerError::RedisAsyncFailure(err) => err.description(),
            BannerError::RedisAsyncSubMessageFailure => "Async Redis message failed",
            BannerError::InvalidRedisConfig => "Can not create RedisStore from invalid config",
            BannerError::AllCacheMissing => "Full cache is misconfigured",
            BannerError::FailedToSerializeItem => "Failed to turn item into json",
            BannerError::UpdatedAtPoisoned => "",
        }
    }
}

impl fmt::Display for BannerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}
