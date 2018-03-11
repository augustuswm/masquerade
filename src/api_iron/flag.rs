use bodyparser;
use iron::prelude::*;
use iron::status;
use router::{Params, Router};
use serde_json;

use std::sync::Arc;

use api::backend::BackendReqExt;
use api::error::APIError;
use error::BannerError;
use flag::{Flag, FlagPath};
use store::ThreadedStore;

struct FlagReq<'a> {
    pub path: FlagPath,
    pub key: Option<&'a str>,
    pub store: Arc<ThreadedStore<FlagPath, Flag, Error = BannerError>>,
}

impl<'a> FlagReq<'a> {
    pub fn from_params(req: &'a mut Request) -> Result<FlagReq<'a>, APIError> {
        let params: &Params = req.extensions
            .get::<Router>()
            .ok_or(APIError::FailedToAccessParams)?;
        let store = req.get_store().ok_or(APIError::FailedToAccessStore)?;

        if let (Some(app), Some(env)) = (params.find("app"), params.find("env")) {
            Ok(FlagReq {
                path: FlagPath {
                    app: app.into(),
                    env: env.into(),
                    path: [app, "$", env].concat(),
                },
                key: params.find("key"),
                store: store,
            })
        } else {
            Err(APIError::FailedToParseParams)
        }
    }
}

pub fn create(req: &mut Request) -> IronResult<Response> {
    if let Ok(Some(flag)) = req.get::<bodyparser::Struct<Flag>>() {
        let flag_req = FlagReq::from_params(req)?;

        if let Ok(Some(_exists)) = flag_req.store.get(&flag_req.path, flag.key()) {
            Err(APIError::AlreadyExists)?
        }

        flag_req
            .store
            .upsert(&flag_req.path, flag.key(), &flag)
            .and_then(|_| Ok(Response::with((status::Created, ""))))
            .map_err(|err| err.into())
    } else {
        Err(APIError::FailedToParseBody)?
    }
}

pub fn read(req: &mut Request) -> IronResult<Response> {
    let flag_req = FlagReq::from_params(req)?;

    if let Some(ref key) = flag_req.key {
        let flag = match flag_req.store.get(&flag_req.path, key) {
            Ok(Some(flag)) => Some(flag),
            _ => None,
        }.ok_or(APIError::FailedToFind)?;

        let stringy_flag = serde_json::to_string(&flag).or(Err(APIError::FailedToSerialize))?;

        Ok(Response::with((status::Ok, stringy_flag)))
    } else {
        Err(APIError::FailedToParseParams)?
    }
}

pub fn update(req: &mut Request) -> IronResult<Response> {
    if let Ok(Some(new_flag)) = req.get::<bodyparser::Struct<Flag>>() {
        let flag_req = FlagReq::from_params(req)?;

        if let Some(ref key) = flag_req.key {
            let mut flag = match flag_req.store.get(&flag_req.path, key) {
                Ok(Some(flag)) => Some(flag),
                _ => None,
            }.ok_or(APIError::FailedToFind)?;

            flag.set_value(new_flag.value());
            flag.toggle(new_flag.is_enabled());

            flag_req
                .store
                .upsert(&flag_req.path, key, &flag)
                .and_then(|_| Ok(Response::with((status::Ok, ""))))
                .map_err(|err| err.into())
        } else {
            Err(APIError::FailedToParseParams)?
        }
    } else {
        Err(APIError::FailedToParseBody)?
    }
}

pub fn delete(req: &mut Request) -> IronResult<Response> {
    let flag_req = FlagReq::from_params(req)?;

    if let Some(ref key) = flag_req.key {
        let flag = match flag_req.store.delete(&flag_req.path, key) {
            Ok(Some(flag)) => Some(flag),
            _ => None,
        }.ok_or(APIError::FailedToFind)?;

        let stringy_flag = serde_json::to_string(&flag).or(Err(APIError::FailedToSerialize))?;

        Ok(Response::with((status::Ok, stringy_flag)))
    } else {
        Err(APIError::FailedToParseParams)?
    }
}

pub fn all(req: &mut Request) -> IronResult<Response> {
    let flag_req = FlagReq::from_params(req)?;

    flag_req
        .store
        .get_all(&flag_req.path)
        .and_then(|flags| {
            let stringy_flags = serde_json::to_string(&flags.values().collect::<Vec<&Flag>>())
                .or(Err(APIError::FailedToSerialize))?;
            Ok(Response::with((status::Ok, stringy_flags)))
        })
        .map_err(|err| err.into())
}
