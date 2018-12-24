use actix_web::http::StatusCode;
use actix_web::*;
use futures::future::Either;
use futures::{future, Future};
use log::error;
use serde_derive::{Deserialize, Serialize};
use serde_json;

use crate::api::error::APIError;
use crate::api::user_req::UserReq;
use crate::api::State;
use crate::user::{User, PATH};

#[derive(Debug, Deserialize, Serialize)]
struct APIUser {
    pub key: String,
    #[serde(skip_serializing)]
    pub secret: Option<String>,
    pub is_admin: bool,
}

impl APIUser {
    pub fn into_user(self) -> Result<User, ()> {
        let APIUser {
            key,
            secret,
            is_admin,
        } = self;

        secret
            .ok_or(())
            .and_then(|secret| User::new(key, secret, is_admin))
    }
}

impl From<&User> for APIUser {
    fn from(user: &User) -> APIUser {
        APIUser {
            key: user.key.clone(),
            secret: None,
            is_admin: user.is_admin(),
        }
    }
}

pub fn read(req: &HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let user_req = match UserReq::from_req(&req) {
        Ok(res) => res,
        Err(err) => return Box::new(future::err(err)),
    };

    Box::new(
        state
            .users()
            .get(&PATH, user_req.key)
            .map_err(APIError::FailedToAccessStore)
            .and_then(|result| {
                if let Some(user) = result {
                    serde_json::to_string(&APIUser::from(&user))
                        .map(|val| val.into())
                        .or(Err(APIError::FailedToSerialize))
                } else {
                    Err(APIError::FailedToFind)
                }
            }),
    )
}

pub fn create(req: &HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();

    Box::new(req.json().from_err().and_then(move |new_user: APIUser| {
        if let Ok(user) = new_user.into_user() {
            Either::A(
                state
                    .users()
                    .get(&PATH, user.key.clone())
                    .map_err(APIError::FailedToAccessStore)
                    .and_then(move |result| {
                        if result.is_some() {
                            Either::A(future::err(APIError::AlreadyExists))
                        } else {
                            Either::B(
                                state
                                    .users()
                                    .upsert(&PATH, user.key.clone(), &user)
                                    .map_err(|_| APIError::FailedToWriteToStore)
                                    .and_then(|_| Ok(HttpResponse::new(StatusCode::CREATED))),
                            )
                        }
                    }),
            )
        } else {
            Either::B(future::err(APIError::InvalidPayload))
        }
    }))
}

pub fn update(req: &HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let user_req = match UserReq::from_req(&req) {
        Ok(res) => res,
        Err(err) => return Box::new(future::err(err)),
    };

    Box::new(req.json().from_err().and_then(move |new_user: APIUser| {
        state
            .users()
            .get(&PATH, user_req.key.clone())
            .map_err(APIError::FailedToAccessStore)
            .and_then(move |result| {
                if let Some(mut user) = result {
                    user.set_key(new_user.key);
                    user.set_admin_status(new_user.is_admin);

                    if let Some(secret) = new_user.secret {
                        if secret.len() != 0 {
                            user.update_secret(&secret)
                        }
                    }

                    Either::A(
                        state
                            .users()
                            .upsert(&PATH, user.key.clone(), &user)
                            .and_then(move |_| state.users().delete(&PATH, user_req.key))
                            .map_err(|_| APIError::FailedToWriteToStore)
                            .and_then(|_| Ok(HttpResponse::new(StatusCode::OK))),
                    )
                } else {
                    Either::B(future::err(APIError::FailedToFind))
                }
            })
    }))
}

pub fn delete(req: &HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let user_req = match UserReq::from_req(&req) {
        Ok(res) => res,
        Err(err) => return Box::new(future::err(err)),
    };

    Box::new(
        state
            .users()
            .delete(&PATH, user_req.key)
            .map_err(|_| APIError::FailedToWriteToStore)
            .and_then(|result| {
                if let Some(user) = result {
                    serde_json::to_string(&APIUser::from(&user))
                        .map(|val| val.into())
                        .or(Err(APIError::FailedToSerialize))
                } else {
                    Err(APIError::FailedToFind)
                }
            }),
    )
}

pub fn all(req: &HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();

    Box::new(
        state
            .users()
            .get_all(&PATH)
            .map_err(APIError::FailedToAccessStore)
            .and_then(|users| {
                let mut user_list = users.values().map(|u| u.into()).collect::<Vec<APIUser>>();
                user_list.as_mut_slice().sort_by(|a, b| a.key.cmp(&b.key));

                Ok(serde_json::to_string(&user_list)
                    .or(Err(APIError::FailedToSerialize))
                    .into())
            }),
    )
}
