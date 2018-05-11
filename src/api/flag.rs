use actix_web::*;
use actix_web::http::StatusCode;
use futures::{future, Future, Stream};
use serde_json;

use std::str;

use api::State;
use api::error::APIError;
use api::flag_req::FlagReq;
use flag::Flag;

pub fn read(req: HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    Box::new(future::ok(()).and_then(move |_| {
        let state = req.state();
        let flag_req = FlagReq::from_req(&req)?;

        if let Some(ref key) = flag_req.key {
            let flag = match state.flags().get(&flag_req.path, key) {
                Ok(Some(flag)) => Some(flag),
                _ => None,
            }.ok_or(APIError::FailedToFind)?;

            Ok(serde_json::to_string(&flag)
                .or(Err(APIError::FailedToSerialize))
                .into())
        } else {
            Err(APIError::FailedToParseParams)
        }
    }))
}

pub fn create(req: HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let flag_req = match FlagReq::from_req(&req) {
        Ok(res) => res,
        Err(err) => return Box::new(future::err(err)),
    };

    req.concat2()
        .from_err()
        .and_then(move |body| {
            if let Ok(flag) = serde_json::from_str::<Flag>(str::from_utf8(&body).unwrap()) {
                // Disallow empty string key
                if flag.key().len() == 0 {
                    Err(APIError::InvalidFlag)?
                }

                if let Ok(Some(_exists)) = state.flags().get(&flag_req.path, flag.key()) {
                    Err(APIError::AlreadyExists)?
                }

                state
                    .flags()
                    .upsert(&flag_req.path, flag.key(), &flag)
                    .and_then(|_| Ok(HttpResponse::new(StatusCode::CREATED)))
                    .map_err(|_| APIError::FailedToWriteToStore)
            } else {
                Err(APIError::FailedToParseBody)
            }
        })
        .responder()
}

pub fn update(req: HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let flag_req = match FlagReq::from_req(&req) {
        Ok(res) => res,
        Err(err) => return Box::new(future::err(err)),
    };

    req.concat2()
        .from_err()
        .and_then(move |body| {
            if let Ok(new_flag) = serde_json::from_str::<Flag>(str::from_utf8(&body).unwrap()) {
                if let Some(ref key) = flag_req.key {
                    let mut flag = match state.flags().get(&flag_req.path, key) {
                        Ok(Some(flag)) => Some(flag),
                        _ => None,
                    }.ok_or(APIError::FailedToFind)?;

                    flag.set_value(new_flag.value());
                    flag.toggle(new_flag.is_enabled());

                    state
                        .flags()
                        .upsert(&flag_req.path, key, &flag)
                        .and_then(|_| Ok(HttpResponse::new(StatusCode::OK)))
                        .map_err(|_| APIError::FailedToWriteToStore)
                } else {
                    Err(APIError::FailedToParseParams)
                }
            } else {
                Err(APIError::FailedToParseBody)
            }
        })
        .responder()
}

pub fn delete(req: HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    Box::new(future::ok(()).and_then(move |_| {
        let state = req.state();
        let flag_req = FlagReq::from_req(&req)?;

        if let Some(ref key) = flag_req.key {
            let flag = state
                .flags()
                .delete(&flag_req.path, key)
                .map_err(|_| APIError::FailedToWriteToStore)
                .and_then(|res| match res {
                    Some(flag) => Ok(flag),
                    None => Err(APIError::FailedToFind),
                })?;

            Ok(serde_json::to_string(&flag)
                .or(Err(APIError::FailedToSerialize))
                .into())
        } else {
            Err(APIError::FailedToParseParams)
        }
    }))
}

pub fn all(req: HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    Box::new(future::ok(()).and_then(move |_| {
        let state = req.state();
        let flag_req = FlagReq::from_req(&req)?;

        state
            .flags()
            .get_all(&flag_req.path)
            .and_then(|flags| {
                let mut flag_list = flags.values().collect::<Vec<&Flag>>();
                flag_list
                    .as_mut_slice()
                    .sort_by(|&a, &b| a.key().cmp(b.key()));

                Ok(serde_json::to_string(&flag_list)
                    .or(Err(APIError::FailedToSerialize))
                    .into())
            })
            .map_err(|_| APIError::FailedToAccessStore)
    }))
}
