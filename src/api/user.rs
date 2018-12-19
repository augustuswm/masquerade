use actix_web::http::StatusCode;
use actix_web::*;
use futures::future::Either;
use futures::{future, Future};
use log::error;
use serde_derive::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

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
    pub fn into_user(&self) -> Result<User, ()> {
        User::new(
            Uuid::new_v4().to_string(),
            self.key.clone(),
            self.secret.clone().unwrap(),
            self.is_admin,
        )
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

pub fn read<'r>(req: &'r HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let user_req = match UserReq::from_req(&req) {
        Ok(res) => res,
        Err(err) => return Box::new(future::err(err)),
    };

    Box::new(
        state
            .users()
            .get(&PATH, &user_req.key)
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

pub fn create<'r>(
    req: &'r HttpRequest<State>,
) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();

    Box::new(req.json()
        .from_err()
        .and_then(move |new_user: APIUser| {
            // Disallow empty string key
            if new_user.key.len() == 0 {
                Either::A(future::err(APIError::InvalidFlag))
            } else {
                Either::B(
                    state.users().get(&PATH, &new_user.key)
                        .map_err(APIError::FailedToAccessStore)
                        .and_then(move |result| {
                            if result.is_some() {
                                Either::A(future::err(APIError::AlreadyExists))
                            } else {
                                if let Ok(user) = new_user.into_user() {
                                    Either::B(
                                        state
                                            .users()
                                            .upsert(&PATH, &new_user.key, &user)
                                            .map_err(|_| APIError::FailedToWriteToStore)
                                            .and_then(|_| {
                                                Ok(HttpResponse::new(StatusCode::CREATED))
                                            })
                                    )
                                } else {
                                    error!("Failed to generate user record for {}. Data could not be persisted", &new_user.key);
                                    Either::A(future::err(APIError::SystemFailure))
                                }
                            }
                        })
                )
            }
        })
    )
}

pub fn update<'r>(
    req: &'r HttpRequest<State>,
) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let user_req = match UserReq::from_req(&req) {
        Ok(res) => res,
        Err(err) => return Box::new(future::err(err)),
    };

    Box::new(req.json().from_err().and_then(move |new_user: APIUser| {
        state
            .users()
            .get(&PATH, &user_req.key)
            .map_err(APIError::FailedToAccessStore)
            .and_then(move |result| {
                if let Some(mut user) = result {
                    user.set_admin_status(new_user.is_admin);

                    Either::A(
                        state
                            .users()
                            .upsert(&PATH, &user_req.key, &user)
                            .map_err(|_| APIError::FailedToWriteToStore)
                            .and_then(|_| Ok(HttpResponse::new(StatusCode::OK))),
                    )
                } else {
                    Either::B(future::err(APIError::FailedToFind))
                }
            })
    }))
}

pub fn delete<'r>(
    req: &'r HttpRequest<State>,
) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let state = req.state().clone();
    let user_req = match UserReq::from_req(&req) {
        Ok(res) => res,
        Err(err) => return Box::new(future::err(err)),
    };

    Box::new(
        state
            .users()
            .delete(&PATH, &user_req.key)
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

pub fn all<'r>(req: &'r HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
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
