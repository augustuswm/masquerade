#[cfg(feature = "redis-backend")]
use redis::{ErrorKind, FromRedisValue, RedisResult, ToRedisArgs, Value as RedisValue};
#[cfg(feature = "dynamo-backend")]
use rusoto_dynamodb::AttributeValue;
#[cfg(feature = "redis-backend")]
use serde_json;

#[cfg(feature = "dynamo-backend")]
use std::collections::HashMap;

#[cfg(feature = "dynamo-backend")]
use storage::dynamo::{DynamoError, FromAttrMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub uuid: String,
    pub key: String,
    pub secret: String,
    is_admin: bool,
}

impl User {
    pub fn new(uuid: String, key: String, secret: String, is_admin: bool) -> User {
        User {
            uuid: uuid,
            key: key,
            secret: secret,
            is_admin: is_admin,
        }
    }

    pub fn is_admin(&self) -> bool {
        self.is_admin
    }
}

#[cfg(feature = "redis-backend")]
impl FromRedisValue for User {
    fn from_redis_value(v: &RedisValue) -> RedisResult<User> {
        match *v {
            RedisValue::Data(ref data) => {
                let data = String::from_utf8(data.clone());

                data.or_else(|_| Err((ErrorKind::TypeError, "Expected utf8 string").into()))
                    .and_then(|ser| {
                        serde_json::from_str(ser.as_str()).or_else(|_| {
                            let err = (ErrorKind::TypeError, "Unable to deserialize json to User");
                            Err(err.into())
                        })
                    })
            }
            _ => {
                let err = (
                    ErrorKind::TypeError,
                    "Recieved non-data type for deserializing",
                );
                Err(err.into())
            }
        }
    }
}

#[cfg(feature = "redis-backend")]
impl<'a> ToRedisArgs for User {
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        let ser = serde_json::to_string(&self);

        vec![
            match ser {
                Ok(json) => json.as_bytes().into(),

                // Because this trait can not normally fail, but json serialization
                // can fail, the failure cause is encoded as a special value that
                // is checked by the store
                Err(_) => "fail".to_string().as_bytes().into(),
            },
        ]
    }
}

#[cfg(feature = "dynamo-backend")]
impl Into<HashMap<String, AttributeValue>> for User {
    fn into(self) -> HashMap<String, AttributeValue> {
        let mut uuid_attr = AttributeValue::default();
        uuid_attr.s = Some(self.uuid);

        let mut key_attr = AttributeValue::default();
        key_attr.s = Some(self.key);

        let mut secret_attr = AttributeValue::default();
        secret_attr.s = Some(self.secret);

        let mut is_admin_attr = AttributeValue::default();
        is_admin_attr.bool = Some(self.is_admin);

        let mut map = HashMap::new();
        map.insert("uuid".into(), uuid_attr);
        map.insert("key".into(), key_attr);
        map.insert("secret".into(), secret_attr);
        map.insert("is_admin".into(), is_admin_attr);

        map
    }
}

#[cfg(feature = "dynamo-backend")]
impl FromAttrMap<User> for User {
    type Error = BannerError;

    fn from_attr_map(mut map: HashMap<String, AttributeValue>) -> Result<User, BannerError> {
        let uuid = map.remove("uuid").and_then(|uuid_data| uuid_data.s);
        let key = map.remove("key").and_then(|key_data| key_data.s);
        let secret = map.remove("secret").and_then(|secret_data| secret_data.s);
        let is_admin = map.remove("is_admin")
            .and_then(|is_admin_data| is_admin_data.bool);

        if let (Some(u), Some(k), Some(s), Some(a)) = (uuid, key, secret, is_admin) {
            Ok(User::new(u, k, s, a))
        } else {
            Err(DynamoError::FailedToParseResponse.into())
        }
    }
}
