use redis_async;
use redis_async::error::Error as RedisAsyncError;
use redis_async::resp::{FromResp, RespValue};
        
// use redis::{ErrorKind, FromRedisValue, RedisResult, ToRedisArgs, Value as RedisValue};
use ring::{digest, pbkdf2};
use serde_json;

static DIGEST_ALG: &'static digest::Algorithm = &digest::SHA256;
const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;
const ITERATIONS: u32 = 5;
const SALT: [u8; 16] = [
    // This value was generated from a secure PRNG.
    0xd6, 0x26, 0x98, 0xda, 0xf4, 0xdc, 0x50, 0x52,
    0x24, 0xf2, 0x27, 0xd1, 0xfe, 0x39, 0x01, 0x8a
];

pub type Credential = [u8; CREDENTIAL_LEN];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub uuid: String,
    pub key: String,
    pub hash: Credential,
    is_admin: bool,
}

impl User {
    pub fn new(uuid: String, key: String, secret: String, is_admin: bool) -> User {
        let hash = User::generate_hash(key.as_str(), secret.as_str());

        User {
            uuid: uuid,
            key: key,
            hash: hash,
            is_admin: is_admin,
        }
    }

    pub fn is_admin(&self) -> bool {
        self.is_admin
    }

    // Example implementation from ring library: https://briansmith.org/rustdoc/ring/pbkdf2/

    pub fn generate_hash(key: &str, secret: &str) -> Credential {
        let salt = User::salt(key);
        let mut to_store: Credential = [0u8; CREDENTIAL_LEN];

        pbkdf2::derive(
            DIGEST_ALG,
            ITERATIONS,
            &salt,
            secret.as_bytes(),
            &mut to_store,
        );

        to_store
    }

    pub fn verify_secret(&self, secret: &str) -> bool {
        let salt = User::salt(&self.key);
        pbkdf2::verify(DIGEST_ALG, ITERATIONS, &salt, secret.as_bytes(), &self.hash).is_ok()
    }

    // The salt should have a user-specific component so that an attacker
    // cannot crack one password for multiple users in the database. It
    // should have a database-unique component so that an attacker cannot
    // crack the same user's password across databases in the unfortunate
    // but common case that the user has used the same password for
    // multiple systems.
    fn salt(key: &str) -> Vec<u8> {
        let mut salt = Vec::with_capacity(SALT.len() + key.as_bytes().len());
        salt.extend(SALT.as_ref());
        salt.extend(key.as_bytes());
        salt
    }
}

redis_conversions!(User);
