use actix_web::dev::AsyncResult;
use actix_web::{FromRequest, HttpRequest, Path};

use crate::api::error::APIError;
use crate::api::State;
use crate::flag::FlagPath;
use crate::user::User;

#[derive(Clone, Debug)]
pub struct FlagReq {
    pub path: FlagPath,
    pub key: Option<String>,
}

impl FlagReq {
    pub fn from_req(req: &HttpRequest<State>) -> Result<FlagReq, APIError> {
        let params = req.match_info();

        if let Some(user) = req.extensions().get::<User>() {
            if let (Some(app), Some(env)) = (params.get("app"), params.get("env")) {
                Ok(FlagReq {
                    path: FlagPath {
                        owner: user.uuid.clone(),
                        app: app.into(),
                        env: env.into(),
                        path: FlagPath::make_path(&user.uuid, app, env),
                    },
                    key: params.get("key").map(|s| s.into()),
                })
            } else {
                Err(APIError::FailedToParseParams)
            }
        } else {
            Err(APIError::Unauthorized)
        }
    }

    pub fn parts(self) -> (FlagPath, Option<String>) {
        (self.path, self.key)
    }
}

impl FromRequest<State> for FlagReq {
    type Config = ();
    type Result = Result<FlagReq, APIError>;

    fn from_request(req: &HttpRequest<State>, _cfg: &Self::Config) -> Self::Result {
        if let Ok(params) = Path::<(String, String, Option<String>)>::extract(req) {
            if let Some(user) = req.extensions().get::<User>() {
                let params = params.clone();
                Ok(FlagReq {
                    path: FlagPath::new(user.uuid.clone(), params.0, params.1),
                    key: params.2,
                })
            } else {
                Err(APIError::Unauthorized)
            }
        } else {
            Err(APIError::FailedToFind)
        }
    }
}

// pub trait FromRequest<S>: Sized {
//     type Config: Default;
//     type Result: Into<AsyncResult<Self>>;
//     fn from_request(req: &HttpRequest<S>, cfg: &Self::Config) -> Self::Result;

//     fn extract(req: &HttpRequest<S>) -> Self::Result { ... }
// }
