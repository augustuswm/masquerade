use serde_derive::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct APIConfig {
    pub jwt_secret: String,
}
