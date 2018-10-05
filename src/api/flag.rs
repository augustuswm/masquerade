use actix_web::*;
use actix_web::http::StatusCode;
use futures::{future, Future, Stream};
use serde_json;

use std::str;

use api::State;
use api::error::APIError;
use api::flag_req::FlagReq;
use flag::Flag;

pub fn read<'r>(req: &'r HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let flag_req = match FlagReq::from_req(&req) {
        Ok(res) => res,
        Err(err) => return Box::new(future::err(err)),
    };

    Box::new(future::ok(()).and_then(move |_| {
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

pub fn create<'r>(req: &'r HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let flag_req = match FlagReq::from_req(&req) {
        Ok(res) => res,
        Err(err) => return Box::new(future::err(err)),
    };

    req.json()
        .from_err()
        .and_then(move |flag: Flag| {
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
        })
        .responder()
}

pub fn update<'r>(req: &'r HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let flag_req = match FlagReq::from_req(&req) {
        Ok(res) => res,
        Err(err) => return Box::new(future::err(err)),
    };

    req.json()
        .from_err()
        .and_then(move |new_flag: Flag| {
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
        })
        .responder()
}

pub fn delete<'r>(req: &'r HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let flag_req = match FlagReq::from_req(&req) {
        Ok(res) => res,
        Err(err) => return Box::new(future::err(err)),
    };

    Box::new(future::ok(()).and_then(move |_| {
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

pub fn all<'r>(req: &'r HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let flag_req = match FlagReq::from_req(&req) {
        Ok(res) => res,
        Err(err) => return Box::new(future::err(err)),
    };

    Box::new(future::ok(()).and_then(move |_| {

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
