use actix_web::http::StatusCode;
use actix_web::*;
use futures::future::Either;
use futures::{future, Future};
use serde_derive::{Deserialize, Serialize};
use serde_json;

use std::str;

use crate::api::error::APIError;
use crate::api::State;
use crate::flag::FlagPath;
use crate::user::User;

const PATH_KEY: &'static str = "paths";

#[derive(Serialize, Deserialize)]
struct FlagPathReq {
    pub app: String,
    pub env: String,
}

pub fn create<'r>(
    req: &'r HttpRequest<State>,
) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let ext = req.extensions();

    if let Some(u) = ext.get::<User>() {
        let user = u.clone();

        Box::new(
            req.json()
                .from_err()
                .and_then(move |f_path_req: FlagPathReq| {
                    let path = PATH_KEY.to_string();
                    let f_path = FlagPath::new(user.uuid.clone(), f_path_req.app, f_path_req.env);

                    state
                        .paths()
                        .get(&path, f_path.as_ref())
                        .map_err(APIError::FailedToAccessStore)
                        .and_then(move |result| {
                            if result.is_some() {
                                Either::A(future::err(APIError::AlreadyExists))
                            } else {
                                Either::B(
                                    state
                                        .paths()
                                        .upsert(&path, f_path.as_ref(), &f_path)
                                        .map_err(|_| APIError::FailedToWriteToStore)
                                        .and_then(|_| Ok(HttpResponse::new(StatusCode::CREATED))),
                                )
                            }
                        })
                }),
        )
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
            .map_err(APIError::FailedToAccessStore)
    }))
}
