use base64::encode;
use redis_async;
use redis_async::error::Error as RedisAsyncError;
use redis_async::resp::{FromResp, RespValue};
use ring::{digest, pbkdf2};
use ring::rand::{SecureRandom, SystemRandom};
use serde_derive::{Deserialize, Serialize};
use serde_json;

static DIGEST_ALG: &'static digest::Algorithm = &digest::SHA256;
const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;
const ITERATIONS: u32 = 5;

pub type Credential = [u8; CREDENTIAL_LEN];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub uuid: String,
    pub key: String,
    pub salt: String,
    pub hash: Credential,
    is_admin: bool,
}

impl User {
    pub fn new(uuid: String, key: String, secret: String, is_admin: bool) -> User {
        let salt = User::salt().unwrap();
        let hash = User::generate_hash(salt.as_bytes(), secret.as_str());

        User {
            uuid: uuid,
            key: key,
            salt: salt,
            hash: hash,
            is_admin: is_admin,
        }
    }

    pub fn is_admin(&self) -> bool {
        self.is_admin
    }

    pub fn generate_hash(salt: &[u8], secret: &str) -> Credential {
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
        // TODO: Combine salt and secret
        pbkdf2::verify(DIGEST_ALG, ITERATIONS, &self.salt.as_bytes(), secret.as_bytes(), &self.hash).is_ok()
    }

    fn salt() -> Result<String, ()> {
        let mut dest: [u8; 16] = [0; 16];
        SystemRandom::new().fill(&mut dest).map_err(|_| ())?;

        Ok(encode(&dest))

        // let mut full_salt = Vec::with_capacity(salt.len() + key.as_bytes().len());
        // full_salt.extend(salt.as_ref());
        // full_salt.extend(key.as_bytes());
        // full_salt
    }
}

redis_conversions!(User);
