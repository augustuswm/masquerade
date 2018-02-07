#[cfg(feature = "redis-backend")]
use redis::RedisError;

#[derive(Debug, PartialEq)]
pub enum BannerError {
    CachePoisonedError,
    FailedToSerializeItem,
    InvalidRedisConfig,
    ItemDoesNotExist,
    #[cfg(feature = "redis-backend")] RedisFailure(RedisError),
}
