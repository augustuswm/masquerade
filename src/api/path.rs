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

pub fn create<'r>(req: &'r HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let ext = req.extensions();
    let u = ext.get::<User>();

    if let Some(u) = ext.get::<User>() {
        let user = u.clone();
        
        req.json()
            .from_err()
            .and_then(move |f_path_req: FlagPathReq| {
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
            })
            .responder()
    } else {
        Box::new(future::err(APIError::Unauthorized))
    }

    
}

pub fn all<'r>(req: &'r HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();

    Box::new(future::ok(()).and_then(move |_| {
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
