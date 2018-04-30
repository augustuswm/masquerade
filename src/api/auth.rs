use actix_web::{HttpRequest, HttpResponse, Result};
use actix_web::middleware::{Middleware, Response, Started};
use base64::decode;
use http::{header, StatusCode};

use std::str;
use std::str::FromStr;

use api::error::APIError;
use api::State;

#[derive(Debug)]
pub struct BasicAuth;

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

impl Middleware<State> for BasicAuth {
    fn start(&self, req: &mut HttpRequest<State>) -> Result<Started> {
        let auth_test = req.clone()
            .headers_mut()
            .get(header::AUTHORIZATION)
            .and_then(|auth| auth.to_str().ok())
            .and_then(|auth| auth.parse::<AuthReq>().ok());

        if let Some(auth_req) = auth_test {
            if let Ok(Some(user)) = req.state()
                .users()
                .get(&"users".to_string(), auth_req.key.as_str())
            {
                if auth_req.secret == user.secret {
                    req.extensions().insert(user);
                    Ok(Started::Done)
                } else {
                    Ok(Started::Response(HttpResponse::new(
                        StatusCode::UNAUTHORIZED,
                        "".into(),
                    )))
                }
            } else {
                Ok(Started::Response(HttpResponse::new(
                    StatusCode::FORBIDDEN,
                    "".into(),
                )))
            }
        } else {
            Ok(Started::Response(HttpResponse::new(
                StatusCode::UNAUTHORIZED,
                "".into(),
            )))
        }
    }

    fn response(&self, _: &mut HttpRequest<State>, resp: HttpResponse) -> Result<Response> {
        Ok(Response::Done(resp))
    }
}
