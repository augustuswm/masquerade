use serde_derive::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct APIConfig {
    pub jwt_secret: String
}

impl APIConfig {
    pub fn new<S>(jwt_secret: S) -> APIConfig where S: Into<String> {
        APIConfig {
            jwt_secret: jwt_secret.into()
        }
    }
}