use std::error::Error;
use std::fmt;

use api::error::APIError;

#[cfg(feature = "redis-backend")]
use redis::RedisError;

#[derive(Debug, PartialEq)]
pub enum BannerError {
    AllCacheMissing,
    APIError(APIError),
    CachePoisonedError,
    FailedToSerializeItem,
    InvalidRedisConfig,
    ItemDoesNotExist,
    #[cfg(feature = "redis-backend")] RedisFailure(RedisError),
}

impl Error for BannerError {
    fn description(&self) -> &str {
        ""
    }
}

impl fmt::Display for BannerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}
