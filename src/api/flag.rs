use actix_web::http::StatusCode;
use actix_web::State as ActixState;
use actix_web::*;
use futures::future::Either;
use futures::{future, Future};
use serde_json;

use crate::api::error::APIError;
use crate::api::flag_req::FlagReq;
use crate::api::state::StoreElements;
use crate::api::State;
use crate::flag::Flag;

pub fn read(
    (flag_req, state): (FlagReq, ActixState<State>),
) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    if let Some(ref key) = flag_req.key {
        Box::new(
            state
                .flags()
                .get(flag_req.path, key.clone())
                .map_err(APIError::FailedToAccessStore)
                .and_then(|result| {
                    if let Some(flag) = result {
                        serde_json::to_string(&flag)
                            .map(|val| val.into())
                            .or(Err(APIError::FailedToSerialize))
                    } else {
                        Err(APIError::FailedToFind(StoreElements::Flag))
                    }
                }),
        )
    } else {
        Box::new(future::err(APIError::FailedToParseParams))
    }
}

pub fn create(
    (flag_req, flag, state): (FlagReq, Json<Flag>, ActixState<State>),
) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    Box::new(
        // Disallow empty string key
        if flag.key().len() == 0 {
            Either::A(future::err(APIError::InvalidFlag))
        } else {
            Either::B(
                state
                    .flags()
                    .get(flag_req.path.clone(), flag.key().to_string())
                    .map_err(APIError::FailedToAccessStore)
                    .and_then(move |result| {
                        if result.is_some() {
                            Either::A(future::err(APIError::AlreadyExists))
                        } else {
                            Either::B(
                                state
                                    .flags()
                                    .upsert(flag_req.path, flag.key().to_string(), &flag)
                                    .map_err(|_| APIError::FailedToWriteToStore)
                                    .and_then(|_| Ok(HttpResponse::new(StatusCode::CREATED))),
                            )
                        }
                    }),
            )
        },
    )
}

pub fn update(
    (flag_req, new_flag, state): (FlagReq, Json<Flag>, ActixState<State>),
) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    Box::new(if let (path, Some(key)) = flag_req.parts() {
        Either::A(
            state
                .flags()
                .get(path.clone(), key.clone())
                .map_err(APIError::FailedToAccessStore)
                .and_then(move |result| {
                    if let Some(mut flag) = result {
                        flag.set_value(new_flag.value());
                        flag.toggle(new_flag.is_enabled());

                        Either::A(
                            state
                                .flags()
                                .upsert(path, key, &flag)
                                .map_err(|_| APIError::FailedToWriteToStore)
                                .and_then(|_| Ok(HttpResponse::new(StatusCode::OK))),
                        )
                    } else {
                        Either::B(future::err(APIError::FailedToFind(StoreElements::Flag)))
                    }
                }),
        )
    } else {
        Either::B(future::err(APIError::FailedToParseParams))
    })
}

pub fn delete(
    (flag_req, state): (FlagReq, ActixState<State>),
) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    Box::new(if let (path, Some(key)) = flag_req.parts() {
        Either::A(
            state
                .flags()
                .delete(path, key)
                .map_err(|_| APIError::FailedToWriteToStore)
                .and_then(|result| {
                    if let Some(flag) = result {
                        serde_json::to_string(&flag)
                            .map(|val| val.into())
                            .or(Err(APIError::FailedToSerialize))
                    } else {
                        Err(APIError::FailedToFind(StoreElements::Flag))
                    }
                }),
        )
    } else {
        Either::B(future::err(APIError::FailedToParseParams))
    })
}

pub fn all(
    (flag_req, state): (FlagReq, ActixState<State>),
) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    Box::new(
        state
            .flags()
            .get_all(flag_req.path)
            .map_err(APIError::FailedToAccessStore)
            .and_then(|flags| {
                let mut flag_list = flags.values().collect::<Vec<&Flag>>();
                flag_list
                    .as_mut_slice()
                    .sort_by(|&a, &b| a.key().cmp(b.key()));

                Ok(serde_json::to_string(&flag_list)
                    .or(Err(APIError::FailedToSerialize))
                    .into())
            }),
    )
}
