use actix_web::http::StatusCode;
use actix_web::State as ActixState;
use actix_web::*;
use futures::future::Either;
use futures::{future, Future};
use serde_derive::{Deserialize, Serialize};
use serde_json;

use crate::api::error::APIError;
use crate::api::state::StoreElements;
use crate::api::State;
use crate::user::{User, PATH};

#[derive(Debug, Deserialize, Serialize)]
pub struct APIUser {
    pub key: String,
    #[serde(skip_serializing)]
    pub secret: Option<String>,
    pub is_admin: bool,
}

impl APIUser {
    pub fn into_user(&self) -> Result<User, ()> {
        let APIUser {
            key,
            secret,
            is_admin,
        } = self;

        match secret {
            Some(ref secret) => User::new(key.to_string(), secret.to_string(), *is_admin),
            None => Err(()),
        }
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

pub fn read(
    (key, state): (Path<String>, ActixState<State>),
) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    Box::new(
        state
            .users()
            .get(&PATH, key.into_inner())
            .map_err(APIError::FailedToAccessStore)
            .and_then(|result| {
                if let Some(user) = result {
                    serde_json::to_string(&APIUser::from(&user))
                        .map(|val| val.into())
                        .or(Err(APIError::FailedToSerialize))
                } else {
                    Err(APIError::FailedToFind(StoreElements::User))
                }
            }),
    )
}

pub fn create(
    (new_user, state): (Json<APIUser>, ActixState<State>),
) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    Box::new(if let Ok(user) = new_user.into_user() {
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
    })
}

pub fn update(
    (key, new_user, state): (Path<String>, Json<APIUser>, ActixState<State>),
) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let key = key.into_inner();

    Box::new(
        state
            .users()
            .get(&PATH, key.clone())
            .map_err(APIError::FailedToAccessStore)
            .and_then(move |result| {
                if let Some(mut user) = result {
                    user.set_key(new_user.key.clone());
                    user.set_admin_status(new_user.is_admin);

                    if let Some(ref secret) = new_user.secret {
                        if secret.len() != 0 {
                            user.update_secret(secret)
                        }
                    }

                    Either::A(
                        state
                            .users()
                            .upsert(&PATH, user.key.clone(), &user)
                            .and_then(move |_| state.users().delete(&PATH, key))
                            .map_err(|_| APIError::FailedToWriteToStore)
                            .and_then(|_| Ok(HttpResponse::new(StatusCode::OK))),
                    )
                } else {
                    Either::B(future::err(APIError::FailedToFind(StoreElements::User)))
                }
            }),
    )
}

pub fn delete(
    (key, state): (Path<String>, ActixState<State>),
) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    Box::new(
        state
            .users()
            .delete(&PATH, key.clone())
            .map_err(|_| APIError::FailedToWriteToStore)
            .and_then(|result| {
                if let Some(user) = result {
                    serde_json::to_string(&APIUser::from(&user))
                        .map(|val| val.into())
                        .or(Err(APIError::FailedToSerialize))
                } else {
                    Err(APIError::FailedToFind(StoreElements::User))
                }
            }),
    )
}

pub fn all(state: ActixState<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
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
