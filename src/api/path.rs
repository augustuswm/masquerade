use actix_web::*;
use actix_web::http::StatusCode;
use futures::{future, Future, Stream};
use serde_json;

use std::str;

use api::State;
use api::error::APIError;
use flag::FlagPath;
use user::User;

const PATH_KEY: &'static str = "paths";

#[derive(Serialize, Deserialize)]
struct FlagPathReq {
    pub app: String,
    pub env: String,
}

pub fn create(req: HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let r2 = req.clone();

    req.concat2()
        .from_err()
        .and_then(move |body| {
            if let Some(user) = r2.extensions().get::<User>() {
                if let Ok(f_path_req) =
                    serde_json::from_str::<FlagPathReq>(str::from_utf8(&body).unwrap())
                {
                    let path = PATH_KEY.to_string();
                    let f_path = FlagPath::new(user.uuid.clone(), f_path_req.app, f_path_req.env);

                    if let Ok(Some(_exists)) = state.paths().get(&path, f_path.as_ref()) {
                        Err(APIError::AlreadyExists)?
                    }

                    state
                        .paths()
                        .upsert(&path, f_path.as_ref(), &f_path)
                        .and_then(|_| Ok(HttpResponse::new(StatusCode::CREATED)))
                        .map_err(|_| APIError::FailedToWriteToStore)
                } else {
                    Err(APIError::FailedToParseBody)
                }
            } else {
                Err(APIError::Unauthorized)
            }
        })
        .responder()
}

pub fn all(req: HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    Box::new(future::ok(()).and_then(move |_| {
        let state = req.state();

        state
            .paths()
            .get_all(&PATH_KEY.to_string())
            .and_then(|paths| {
                Ok(
                    serde_json::to_string(&paths.values().collect::<Vec<&FlagPath>>())
                        .or(Err(APIError::FailedToSerialize))
                        .into(),
                )
            })
            .map_err(|_| APIError::FailedToAccessStore)
    }))
}
