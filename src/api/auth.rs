use actix_web::{HttpRequest, HttpResponse, Result};
use actix_web::middleware::{Middleware, Response, Started};
use base64::decode;
use futures::{future, Future};
use http::{header, StatusCode};
use jsonwebtoken::{decode as jwt_decode, encode as jwt_encode, Header, Validation};

use std::env;
use std::str;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use api::error::APIError;
use api::State;
use api::state::AsyncUserStore;
use error::Error;
use user::User;

static APP_NAME: &'static str = "masquerade";

#[derive(Debug)]
pub struct BasicAuth;

#[derive(Debug)]
pub struct UrlAuth;

#[derive(Debug)]
pub struct JWTAuth;

#[derive(Debug)]
pub struct RequireUser;

pub struct AuthReq {
    key: String,
    secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    iss: String,
    iat: u64,
    exp: u64,
    nbf: u64,
    cid: String,
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

pub fn authenticate<'r>(req: &'r HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    let ext = req.extensions();

    Box::new(future::result(ext.get::<User>().ok_or(APIError::Unauthorized).and_then(|user| {
        SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| APIError::ConfigFailure).and_then(|time| {
            let now = time.as_secs();
            let allowed_for = 24 * 60 * 60;

            let claims = Claims {
                iss: APP_NAME.to_string(),
                iat: now,
                exp: now + allowed_for,
                nbf: now,
                cid: user.key.clone()
            };

            env::var("JWT_SECRET")
                .map_err(|_| APIError::ConfigFailure)
                .and_then(|secret| {
                    Ok(jwt_encode(&Header::default(), &claims, secret.as_ref())?.into())
                })
        })
    })))
}

fn find_user(key: &str, store: &AsyncUserStore) -> impl Future<Item = Option<User>, Error = Error> {
    store.get(&"users".to_string(), key)
}

fn verify_auth(auth: AuthReq, store: &AsyncUserStore) -> impl Future<Item = Option<User>, Error = Error> {
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
                Ok(Started::Done)
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

impl Middleware<State> for JWTAuth {
    fn start(&self, req: &HttpRequest<State>) -> Result<Started> {
        
        // If the user was already authenticated by some other means,
        // use the already set user
        if req.extensions().get::<User>().is_some() {
            Ok(Started::Done)
        } else {
            let mut validation = Validation::default();
            validation.iss = Some(APP_NAME.to_string());

            let jwt = req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth| auth.to_str().ok())
                .and_then(|auth| {
                    env::var("JWT_SECRET").ok().and_then(|secret| {
                        jwt_decode::<Claims>(&auth[6..], secret.as_ref(), &validation).ok()
                    })
                });

            if let Some(token) = jwt {
                let req = req.clone();

                Ok(Started::Future(Box::new(
                    find_user(&token.claims.cid, req.state().users())
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
                )))
            } else {
                Ok(Started::Done)
            }
        }
    }

    fn response(&self, _: &HttpRequest<State>, resp: HttpResponse) -> Result<Response> {
        Ok(Response::Done(resp))
    }
}

impl Middleware<State> for RequireUser {
    fn start(&self, req: &HttpRequest<State>) -> Result<Started> {
        if req.extensions().get::<User>().is_some() {
            Ok(Started::Done)
        } else {
            Ok(Started::Response(HttpResponse::new(StatusCode::UNAUTHORIZED)))
        }
    }

    fn response(&self, _: &HttpRequest<State>, resp: HttpResponse) -> Result<Response> {
        Ok(Response::Done(resp))
    }
}