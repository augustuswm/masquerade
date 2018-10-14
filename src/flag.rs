use redis_async;
use redis_async::error::Error as RedisAsyncError;
use redis_async::resp::{FromResp, RespValue};
use redis::{ErrorKind, FromRedisValue, RedisResult, ToRedisArgs, Value as RedisValue};
use serde_json;

use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use error::BannerError;

const PATH_SEP: &'static str = ":";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagPath {
    pub owner: String,
    pub app: String,
    pub env: String,
    pub path: String,
}

impl FlagPath {
    pub fn new<T, S, U>(owner: T, app: S, env: U) -> FlagPath
    where
        T: Into<String>,
        S: Into<String>,
        U: Into<String>,
    {
        let o = owner.into();
        let a = app.into();
        let e = env.into();
        let path = FlagPath::make_path(&o, &a, &e);

        FlagPath {
            owner: o,
            app: a,
            env: e,
            path: path,
        }
    }

    pub fn make_path<T, S, U>(owner: T, app: S, env: U) -> String
    where
        T: AsRef<str>,
        S: AsRef<str>,
        U: AsRef<str>,
    {
        [
            owner.as_ref(),
            PATH_SEP,
            app.as_ref(),
            PATH_SEP,
            env.as_ref(),
        ].concat()
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
        let parts: Vec<&str> = s.split(PATH_SEP).collect();

        if parts.len() == 3 {
            Ok(FlagPath::new(parts[0], parts[1], parts[2]))
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

impl<'a> ToRedisArgs for Flag {
    fn write_redis_args(&self, out: &mut Vec<Vec<u8>>) {
        let ser = serde_json::to_string(&self);

        out.push(
            match ser {
                Ok(json) => json.as_bytes().into(),

                // Because this trait can not normally fail, but json serialization
                // can fail, the failure cause is encoded as a special value that
                // is checked by the store
                Err(_) => "fail".to_string().as_bytes().into(),
            },
        )
    }
}

impl FromResp for Flag {
    fn from_resp_int(resp: RespValue) -> Result<Flag, RedisAsyncError> {
        match resp {
            RespValue::BulkString(ref bytes) => {
                serde_json::from_str(&String::from_utf8_lossy(bytes)).or_else(|_| {
                    // let err = (ErrorKind::TypeError, "Unable to deserialize json to Flag");
                    Err(redis_async::error::resp("Cannot convert into a Flag", redis_async::resp::RespValue::BulkString(bytes.to_owned())))
                })
            },
            RespValue::SimpleString(ref string) => {
                serde_json::from_str(string.as_str()).or_else(|_| {
                    // let err = (ErrorKind::TypeError, "Unable to deserialize json to Flag");
                    Err(redis_async::error::resp("Cannot convert into a Flag", resp.to_owned()))
                })
            },
            _ => Err(redis_async::error::resp("Cannot convert into a Flag", resp)),
        }
    }
}

impl Into<RespValue> for Flag {
    fn into(self: Self) -> RespValue {
        let ser = serde_json::to_string(&self);
        RespValue::BulkString(ser.unwrap().as_bytes().to_vec())
    }
}

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

impl<'a> ToRedisArgs for FlagPath {
    fn write_redis_args(&self, out: &mut Vec<Vec<u8>>) {
        let ser = serde_json::to_string(&self);

        out.push(
            match ser {
                Ok(json) => json.as_bytes().into(),

                // Because this trait can not normally fail, but json serialization
                // can fail, the failure cause is encoded as a special value that
                // is checked by the store
                Err(_) => "fail".to_string().as_bytes().into(),
            },
        )
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
