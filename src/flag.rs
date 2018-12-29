use redis_async::error::Error as RedisAsyncError;
use redis_async::resp::{FromResp, RespValue};
use serde_derive::{Deserialize, Serialize};
use serde_json;

use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::Error;

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
        ]
        .concat()
    }
}

impl AsRef<str> for FlagPath {
    fn as_ref(&self) -> &str {
        self.path.as_str()
    }
}

impl FromStr for FlagPath {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(PATH_SEP).collect();

        if parts.len() == 3 {
            Ok(FlagPath::new(parts[0], parts[1], parts[2]))
        } else {
            Err(Error::FailedToParsePath)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Flag {
    key: String,
    value: FlagValue,
    version: u64,
    enabled: bool,
    #[serde(default = "current_time")]
    created: u64,
    #[serde(default = "current_time")]
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

redis_conversions!(Flag);
redis_conversions!(FlagPath);

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
