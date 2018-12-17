use actix_web::{HttpRequest};

use crate::api::error::APIError;
use crate::api::State;
use crate::flag::FlagPath;
use crate::user::User;

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