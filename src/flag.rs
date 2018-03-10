#[cfg(feature = "mongo-backend")]
use bson;
#[cfg(feature = "redis-backend")]
use redis::{ErrorKind, FromRedisValue, RedisResult, ToRedisArgs, Value as RedisValue};
#[cfg(feature = "dynamo-backend")]
use rusoto_dynamodb::AttributeValue;
#[cfg(feature = "redis-backend")]
use serde_json;

#[cfg(feature = "dynamo-backend")]
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use error::BannerError;
#[cfg(feature = "dynamo-backend")]
use storage::dynamo::{DynamoError, FromAttrMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagPath {
    pub app: String,
    pub env: String,
    pub path: String,
}

impl FlagPath {
    pub fn new<T, S>(app: T, env: S) -> FlagPath
    where
        T: Into<String>,
        S: Into<String>,
    {
        let a = app.into();
        let e = env.into();
        let path = [a.as_str(), "$", e.as_str()].concat();

        FlagPath {
            app: a,
            env: e,
            path: path,
        }
    }
}

impl AsRef<str> for FlagPath {
    fn as_ref(&self) -> &str {
        self.path.as_str()
    }
}

impl FromStr for FlagPath {
    type Err = BannerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("$").collect();

        if parts.len() == 2 {
            Ok(FlagPath::new(parts[0], parts[1]))
        } else {
            Err(BannerError::FailedToParsePath)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Flag {
    key: String,
    value: FlagValue,
    #[cfg_attr(feature = "mongo-backend", serde(with = "bson::compat::u2f"))] version: u64,
    enabled: bool,
    #[serde(default = "current_time")]
    #[cfg_attr(feature = "mongo-backend", serde(with = "bson::compat::u2f"))]
    created: u64,
    #[serde(default = "current_time")]
    #[cfg_attr(feature = "mongo-backend", serde(with = "bson::compat::u2f"))]
    updated: u64,
}

fn current_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

impl Flag {
    pub fn new<S>(key: S, value: FlagValue, version: u64, enabled: bool) -> Flag
    where
        S: Into<String>,
    {
        let created = current_time();

        Flag {
            key: key.into(),
            value: value,
            version: version,
            enabled: enabled,
            created: created,
            updated: created,
        }
    }

    pub fn eval(&self) -> Option<&FlagValue> {
        if self.enabled {
            Some(&self.value)
        } else {
            None
        }
    }

    pub fn value(&self) -> &FlagValue {
        &self.value
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_ver(&self, ver: u64) -> bool {
        self.version == ver
    }

    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    pub fn set_value(&mut self, val: &FlagValue) {
        let new_val = val.clone();

        if self.value != new_val {
            self.version = self.version + 1;
            self.value = new_val;
            self.updated = current_time();
        }
    }

    pub fn toggle(&mut self, state: bool) {
        if self.enabled != state {
            self.enabled = !self.enabled;
            self.updated = current_time();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FlagValue {
    Bool(bool),
}

// Backend Impls

#[cfg(feature = "redis-backend")]
impl FromRedisValue for Flag {
    fn from_redis_value(v: &RedisValue) -> RedisResult<Flag> {
        match *v {
            RedisValue::Data(ref data) => {
                let data = String::from_utf8(data.clone());

                data.or_else(|_| Err((ErrorKind::TypeError, "Expected utf8 string").into()))
                    .and_then(|ser| {
                        serde_json::from_str(ser.as_str()).or_else(|_| {
                            let err = (ErrorKind::TypeError, "Unable to deserialize json to Flag");
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
impl<'a> ToRedisArgs for Flag {
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

#[cfg(feature = "redis-backend")]
impl FromRedisValue for FlagPath {
    fn from_redis_value(v: &RedisValue) -> RedisResult<FlagPath> {
        match *v {
            RedisValue::Data(ref data) => {
                let data = String::from_utf8(data.clone());

                data.or_else(|_| Err((ErrorKind::TypeError, "Expected utf8 string").into()))
                    .and_then(|ser| {
                        serde_json::from_str(ser.as_str()).or_else(|_| {
                            let err = (ErrorKind::TypeError, "Unable to deserialize json to Path");
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
impl<'a> ToRedisArgs for FlagPath {
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
impl Into<HashMap<String, AttributeValue>> for Flag {
    fn into(self) -> HashMap<String, AttributeValue> {
        let mut key_attr = AttributeValue::default();
        key_attr.s = Some(self.key);

        let mut value_attr = AttributeValue::default();
        match self.value {
            FlagValue::Bool(true) => {
                value_attr.bool = Some(true);
            }
            _ => {
                value_attr.bool = Some(false);
            }
        }

        let mut version_attr = AttributeValue::default();
        version_attr.n = Some(self.version.to_string());

        let mut enabled_attr = AttributeValue::default();
        enabled_attr.bool = Some(self.enabled);

        let mut created_attr = AttributeValue::default();
        created_attr.n = Some(self.created.to_string());

        let mut updated_attr = AttributeValue::default();
        updated_attr.n = Some(self.updated.to_string());

        let mut map = HashMap::new();
        map.insert("key".into(), key_attr);
        map.insert("value".into(), value_attr);
        map.insert("version".into(), version_attr);
        map.insert("enabled".into(), enabled_attr);
        map.insert("created".into(), created_attr);
        map.insert("updated".into(), updated_attr);

        map
    }
}

#[cfg(feature = "dynamo-backend")]
impl FromAttrMap<Flag> for Flag {
    type Error = BannerError;

    fn from_attr_map(mut map: HashMap<String, AttributeValue>) -> Result<Flag, BannerError> {
        let key = map.remove("key").and_then(|key_data| match key_data.s {
            Some(key) => Some(key),
            None => None,
        });
        let value = map.get("value").and_then(|value_data| value_data.bool);
        let version = map.get("version")
            .and_then(|version_data| match version_data.n {
                Some(ref version) => version.parse::<u64>().ok(),
                None => None,
            });
        let enabled = map.get("enabled")
            .and_then(|enabled_data| enabled_data.bool);
        let created = map.get("created")
            .and_then(|created_data| match created_data.n {
                Some(ref created) => created.parse::<u64>().ok(),
                None => None,
            });
        let updated = map.get("updated")
            .and_then(|updated_data| match updated_data.n {
                Some(ref updated) => updated.parse::<u64>().ok(),
                None => None,
            });

        if let (Some(k), Some(vl), Some(vr), Some(e), Some(c), Some(u)) =
            (key, value, version, enabled, created, updated)
        {
            Ok(Flag {
                key: k,
                value: FlagValue::Bool(vl),
                version: vr,
                enabled: e,
                created: c,
                updated: u,
            })
        } else {
            Err(DynamoError::FailedToParseResponse.into())
        }
    }
}

#[cfg(feature = "dynamo-backend")]
impl Into<HashMap<String, AttributeValue>> for FlagPath {
    fn into(self) -> HashMap<String, AttributeValue> {
        let mut app_attr = AttributeValue::default();
        app_attr.s = Some(self.app);

        let mut env_attr = AttributeValue::default();
        env_attr.s = Some(self.env);

        let mut path_attr = AttributeValue::default();
        path_attr.s = Some(self.path);

        let mut map = HashMap::new();
        map.insert("app".into(), app_attr);
        map.insert("env".into(), env_attr);
        map.insert("path".into(), path_attr);

        map
    }
}

#[cfg(feature = "dynamo-backend")]
impl FromAttrMap<FlagPath> for FlagPath {
    type Error = BannerError;

    fn from_attr_map(mut map: HashMap<String, AttributeValue>) -> Result<FlagPath, BannerError> {
        let app = map.remove("app").and_then(|app_data| match app_data.s {
            Some(app) => Some(app),
            None => None,
        });
        let env = map.remove("env").and_then(|env_data| match env_data.s {
            Some(env) => Some(env),
            None => None,
        });

        if let (Some(a), Some(e)) = (app, env) {
            Ok(FlagPath::new(a, e))
        } else {
            Err(DynamoError::FailedToParseResponse.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_returns_some_if_enabled() {
        let f = Flag::new("key-string", FlagValue::Bool(true), 1, true);
        assert_eq!(f.eval(), Some(&FlagValue::Bool(true)));
    }

    #[test]
    fn test_returns_none_if_disabled() {
        let f = Flag::new("key-string", FlagValue::Bool(true), 1, false);
        assert_eq!(f.eval(), None);
    }

    #[test]
    fn test_returns_enabled_status() {
        let f1 = Flag::new("key-string", FlagValue::Bool(true), 1, true);
        let f2 = Flag::new("key-string", FlagValue::Bool(true), 1, false);
        assert_eq!(f1.is_enabled(), true);
        assert_eq!(f2.is_enabled(), false);
    }

    #[test]
    fn test_checks_version() {
        let f = Flag::new("key-string", FlagValue::Bool(true), 1, true);
        assert_eq!(f.is_ver(1), true);
        assert_eq!(f.is_ver(2), false);
    }
}
