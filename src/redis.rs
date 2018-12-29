use hashbrown::HashMap;
use redis_async::error;
use redis_async::error::Error;
use redis_async::resp::{FromResp, RespValue};

use std::cmp::Eq;
use std::hash::Hash;

#[derive(Debug)]
pub struct RedisHashMap<K, V>(HashMap<K, V>)
where
    K: Eq + Hash;

impl<K, V> RedisHashMap<K, V>
where
    K: Eq + Hash,
{
    pub fn into_hashmap(self) -> HashMap<K, V> {
        self.0
    }
}

impl<K, V> FromResp for RedisHashMap<K, V>
where
    K: FromResp + Eq + Hash,
    V: FromResp,
{
    fn from_resp_int(resp: RespValue) -> Result<RedisHashMap<K, V>, Error> {
        match resp {
            RespValue::Array(ary) => {
                let mut map = HashMap::new();
                let mut items = ary.into_iter();

                while let Some(k) = items.next() {
                    let key = K::from_resp(k)?;
                    let value = V::from_resp(items.next().ok_or(error::resp(
                        "Cannot convert an odd number of elements into a hashmap",
                        "".into(),
                    ))?)?;

                    map.insert(key, value);
                }

                Ok(RedisHashMap(map))
            }
            _ => Err(error::resp("Cannot be converted into a hashmap", resp)),
        }
    }
}

macro_rules! redis_conversions {
    ($struct:ident) => {
        impl FromResp for $struct {
            fn from_resp_int(resp: RespValue) -> Result<$struct, RedisAsyncError> {
                match resp {
                    RespValue::BulkString(ref bytes) => {
                        serde_json::from_str(&String::from_utf8_lossy(bytes)).or_else(|_| {
                            Err(redis_async::error::resp(
                                "Cannot convert into a $struct",
                                redis_async::resp::RespValue::BulkString(bytes.to_owned()),
                            ))
                        })
                    }
                    RespValue::SimpleString(ref string) => serde_json::from_str(string.as_str())
                        .or_else(|_| {
                            Err(redis_async::error::resp(
                                "Cannot convert into a $struct",
                                resp.to_owned(),
                            ))
                        }),
                    _ => Err(redis_async::error::resp(
                        "Cannot convert into a $struct",
                        resp,
                    )),
                }
            }
        }

        impl Into<RespValue> for $struct {
            fn into(self: Self) -> RespValue {
                let res = serde_json::to_string(&self);

                match res {
                    Ok(ser) => RespValue::BulkString(ser.as_bytes().to_vec()),
                    Err(_) => RespValue::BulkString(crate::backend_async::FAIL.to_vec()),
                }
            }
        }
    };
}
