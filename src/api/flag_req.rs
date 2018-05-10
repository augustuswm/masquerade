use actix_web::{HttpRequest};

use api::error::APIError;
use api::State;
use flag::FlagPath;
use user::User;

pub struct FlagReq {
    pub path: FlagPath,
    pub key: Option<String>,
}

impl FlagReq {
    pub fn from_req(req: &HttpRequest<State>) -> Result<FlagReq, APIError> {
        let params = req.match_info();

        if let Some(user) = req.clone().extensions().get::<User>() {
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
}