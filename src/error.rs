#[cfg(feature = "redis-backend")]
use redis::RedisError;
use serde_json::Error as SerdeError;

use std::error::Error;
use std::fmt;

// use api::error::APIError;
#[cfg(feature = "dynamo-backend")]
use storage::dynamo::DynamoError;
#[cfg(feature = "mongo-backend")]
use storage::mongo::MongoError;

#[derive(Debug)]
pub enum BannerError {
    // APIError(APIError),
    CachePoisonedError,
    FailedToParsePath,
    #[cfg(feature = "dynamo-backend")] DynamoFailure(DynamoError),
    #[cfg(feature = "mongo-backend")] MongoFailure(MongoError),
    #[cfg(feature = "redis-backend")] RedisFailure(RedisError),

    #[cfg(feature = "redis-backend")] InvalidRedisConfig,
    AllCacheMissing,
    FailedToSerializeItem,
    UpdatedAtPoisoned
}

#[cfg(feature = "dynamo-backend")]
impl From<DynamoError> for BannerError {
    fn from(err: DynamoError) -> BannerError {
        BannerError::DynamoFailure(err)
    }
}

#[cfg(feature = "redis-backend")]
impl From<RedisError> for BannerError {
    fn from(err: RedisError) -> BannerError {
        BannerError::RedisFailure(err)
    }
}

#[cfg(feature = "mongo-backend")]
impl From<MongoError> for BannerError {
    fn from(err: MongoError) -> BannerError {
        BannerError::MongoFailure(err)
    }
}

impl From<SerdeError> for BannerError {
    fn from(err: SerdeError) -> BannerError {
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
