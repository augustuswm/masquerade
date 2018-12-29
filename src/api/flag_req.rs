use actix_web::{FromRequest, HttpRequest, Path};
use serde_derive::{Deserialize, Serialize};

use crate::api::error::APIError;
use crate::api::State;
use crate::flag::FlagPath;
use crate::user::User;

#[derive(Clone, Serialize, Deserialize)]
struct FlagCreateReq {
    pub app: String,
    pub env: String,
}

impl FlagCreateReq {
    pub fn to_flag_req(self, user: &User) -> FlagReq {
        let FlagCreateReq { app, env } = self;
        FlagReq {
            path: FlagPath::new(user.uuid.clone(), app, env),
            key: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct FlagTouchReq {
    pub app: String,
    pub env: String,
    pub key: String,
}

impl FlagTouchReq {
    pub fn to_flag_req(self, user: &User) -> FlagReq {
        let FlagTouchReq { app, env, key } = self;
        FlagReq {
            path: FlagPath::new(user.uuid.clone(), app, env),
            key: Some(key),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FlagReq {
    pub path: FlagPath,
    pub key: Option<String>,
}

impl FlagReq {
    pub fn parts(self) -> (FlagPath, Option<String>) {
        (self.path, self.key)
    }
}

impl FromRequest<State> for FlagReq {
    type Config = ();
    type Result = Result<FlagReq, APIError>;

    fn from_request(req: &HttpRequest<State>, _cfg: &Self::Config) -> Self::Result {
        if let Ok(params) = Path::<FlagTouchReq>::extract(req) {
            if let Some(user) = req.extensions().get::<User>() {
                Ok(params.clone().to_flag_req(user))
            } else {
                Err(APIError::Unauthorized)
            }
        } else if let Ok(params) = Path::<FlagCreateReq>::extract(req) {
            if let Some(user) = req.extensions().get::<User>() {
                Ok(params.clone().to_flag_req(user))
            } else {
                Err(APIError::Unauthorized)
            }
        } else {
            Err(APIError::FailedToParseParams)
        }
    }
}

// pub trait FromRequest<S>: Sized {
//     type Config: Default;
//     type Result: Into<AsyncResult<Self>>;
//     fn from_request(req: &HttpRequest<S>, cfg: &Self::Config) -> Self::Result;

//     fn extract(req: &HttpRequest<S>) -> Self::Result { ... }
// }
