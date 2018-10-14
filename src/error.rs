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
    RedisAsyncSubConnectionFailure(RedisAsyncError),
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
        BannerError::RedisAsyncSubConnectionFailure(err)
    }
}

impl From<SerdeError> for BannerError {
    fn from(_: SerdeError) -> BannerError {
        BannerError::FailedToSerializeItem
    }
}

impl Error for BannerError {
    fn description(&self) -> &str {
        ""
    }
}

impl fmt::Display for BannerError {
    fn fmt(&self, _: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}
