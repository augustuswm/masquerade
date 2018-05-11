use actix_web::{HttpRequest, HttpResponse, Result};
use actix_web::middleware::{Middleware, Response, Started};
use base64::decode;
use http::{header, StatusCode};

use std::str;
use std::str::FromStr;

use api::error::APIError;
use api::State;
use api::state::UserStore;
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

fn find_user(auth: &AuthReq, store: &Box<UserStore>) -> Option<User> {
    store.get(&"users".to_string(), auth.key.as_str()).unwrap_or(None)
}

fn verifiy_auth(auth: &AuthReq, store: &Box<UserStore>) -> Option<User> {
    find_user(auth, store).and_then(|user| {
        if user.verify_secret(&auth.secret) {
            Some(user)
        } else {
            None
        }
    })
}

fn handle_auth(auth: &AuthReq, req: &mut HttpRequest<State>) -> Started {
    if let Some(user) = verifiy_auth(auth, req.state().users()) {
        req.extensions_mut().insert(user);
        Started::Done
    } else {
        Started::Response(HttpResponse::new(
            StatusCode::UNAUTHORIZED
        ))
    }
}

impl Middleware<State> for BasicAuth {
    fn start(&self, req: &mut HttpRequest<State>) -> Result<Started> {

        // If the user was already authenticated by some other means,
        // use the already set user
        if req.clone().extensions().get::<User>().is_some() {
            Ok(Started::Done)
        } else {
            let auth_test = req
                .headers_mut()
                .get(header::AUTHORIZATION)
                .and_then(|auth| auth.to_str().ok())
                .and_then(|auth| auth[6..].parse::<AuthReq>().ok());

            if let Some(auth_req) = auth_test {
                Ok(handle_auth(&auth_req, req))
            } else {
                println!("Denied access to due to missing credentials");

                Ok(Started::Response(HttpResponse::new(
                    StatusCode::UNAUTHORIZED
                )))
            }
        }
    }

    fn response(&self, _: &mut HttpRequest<State>, resp: HttpResponse) -> Result<Response> {
        Ok(Response::Done(resp))
    }
}

impl Middleware<State> for UrlAuth {
    fn start(&self, req: &mut HttpRequest<State>) -> Result<Started> {
        
        // If the user was already authenticated by some other means,
        // use the already set user
        if req.clone().extensions().get::<User>().is_some() {
            Ok(Started::Done)
        } else {
            let auth_test = req.query().get("auth").and_then(|auth| auth.parse::<AuthReq>().ok());

            if let Some(auth_req) = auth_test {
                Ok(handle_auth(&auth_req, req))
            } else {
                Ok(Started::Done)
            }
        }
    }

    fn response(&self, _: &mut HttpRequest<State>, resp: HttpResponse) -> Result<Response> {
        Ok(Response::Done(resp))
    }
}
