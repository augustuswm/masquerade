use actix_web::{HttpRequest, HttpResponse, Result};
use actix_web::middleware::{Middleware, Response, Started};
use base64::decode;
use futures::{future, Future};
use http::{header, StatusCode};

use std::str;
use std::str::FromStr;

use api::error::APIError;
use api::State;
use api::state::AsyncUserStore;
use error::BannerError;
use user::User;

#[derive(Debug)]
pub struct BasicAuth;

#[derive(Debug)]
pub struct UrlAuth;

pub struct AuthReq {
    key: String,
    secret: String,
}

impl FromStr for AuthReq {
    type Err = APIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = decode(s).or(Err(APIError::FailedToParseAuth))?;
        let parts = str::from_utf8(&decoded[..])
            .or(Err(APIError::FailedToParseAuth))?
            .split(":")
            .collect::<Vec<&str>>();

        Ok(AuthReq {
            key: parts[0].to_string(),
            secret: parts[1].to_string(),
        })
    }
}

fn find_user(key: &str, store: &AsyncUserStore) -> impl Future<Item = Option<User>, Error = BannerError> {
    store.get(&"users".to_string(), key)
}

fn verify_auth(auth: AuthReq, store: &AsyncUserStore) -> impl Future<Item = Option<User>, Error = BannerError> {
    find_user(auth.key.as_str(), store).and_then(move |user| {
        let u = if let Some(user) = user {
            if user.verify_secret(&auth.secret) {
                Some(user)
            } else {
                None
            }
        } else {
            None
        };

        future::ok(u)
    })
}

fn handle_auth(auth: AuthReq, req: &HttpRequest<State>) -> Started {
    let req = req.clone();
    
    Started::Future(
        Box::new(verify_auth(auth, req.state().users())
            .map_err(APIError::FailedToAccessStore)
            .map_err(|e| e.into())
            .and_then(move |user| {
                if let Some(user) = user {
                    req.extensions_mut().insert(user);
                    future::ok(None)
                } else {
                    future::ok(Some(HttpResponse::new(
                        StatusCode::UNAUTHORIZED
                    )))
                }
            })
        )
    )
}

impl Middleware<State> for BasicAuth {
    fn start(&self, req: &HttpRequest<State>) -> Result<Started> {
        // let mut req = req.clone();

        // If the user was already authenticated by some other means,
        // use the already set user
        if req.extensions().get::<User>().is_some() {
            Ok(Started::Done)
        } else {
            let auth_test = req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth| auth.to_str().ok())
                .and_then(|auth| auth[6..].parse::<AuthReq>().ok());

            if let Some(auth_req) = auth_test {
                Ok(handle_auth(auth_req, req))
            } else {
                println!("Denied access to due to missing credentials");

                Ok(Started::Response(HttpResponse::new(
                    StatusCode::UNAUTHORIZED
                )))
            }
        }
    }

    fn response(&self, _: &HttpRequest<State>, resp: HttpResponse) -> Result<Response> {
        Ok(Response::Done(resp))
    }
}

impl Middleware<State> for UrlAuth {
    fn start(&self, req: &HttpRequest<State>) -> Result<Started> {
        
        // If the user was already authenticated by some other means,
        // use the already set user
        if req.extensions().get::<User>().is_some() {
            Ok(Started::Done)
        } else {
            let auth_test = req.query().get("auth").and_then(|auth| auth.parse::<AuthReq>().ok());

            if let Some(auth_req) = auth_test {
                Ok(handle_auth(auth_req, req))
            } else {
                Ok(Started::Done)
            }
        }
    }

    fn response(&self, _: &HttpRequest<State>, resp: HttpResponse) -> Result<Response> {
        Ok(Response::Done(resp))
    }
}
