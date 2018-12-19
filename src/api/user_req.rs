use actix_web::HttpRequest;

use crate::api::error::APIError;
use crate::api::State;

pub struct UserReq {
    pub key: String,
}

impl UserReq {
    pub fn from_req(req: &HttpRequest<State>) -> Result<UserReq, APIError> {
        let params = req.match_info();

        if let Some(key) = params.get("key") {
            Ok(UserReq {
                key: key.to_string(),
            })
        } else {
            Err(APIError::FailedToParseParams)
        }
    }
}
