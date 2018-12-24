use actix_web::error::{JsonPayloadError, PayloadError};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use futures::{future, Future};
use jsonwebtoken::errors::Error as JWTError;
use serde_json::Error as SerdeError;

use std::error::Error as StdError;
use std::fmt;

use crate::error::Error;

#[derive(Debug)]
pub enum APIError {
    AlreadyExists,
    ConfigFailure,
    FailedToAccessStore(Error),
    FailedToFind,
    FailedToParseAuth,
    FailedToParseBody,
    FailedToParseParams,
    FailedToSerialize,
    FailedToWriteToStore,
    JWTError(JWTError),
    InvalidFlag,
    InvalidPayload,
    Unauthorized,
    SystemFailure,
}

impl APIError {
    pub fn status(&self) -> StatusCode {
        match self {
            &APIError::AlreadyExists => StatusCode::CONFLICT,
            &APIError::ConfigFailure => StatusCode::INTERNAL_SERVER_ERROR,
            &APIError::FailedToAccessStore(_) => StatusCode::INTERNAL_SERVER_ERROR,
            &APIError::FailedToFind => StatusCode::NOT_FOUND,
            &APIError::FailedToParseAuth => StatusCode::BAD_REQUEST,
            &APIError::FailedToParseBody => StatusCode::BAD_REQUEST,
            &APIError::FailedToParseParams => StatusCode::BAD_REQUEST,
            &APIError::FailedToSerialize => StatusCode::INTERNAL_SERVER_ERROR,
            &APIError::FailedToWriteToStore => StatusCode::INTERNAL_SERVER_ERROR,
            &APIError::JWTError(_) => StatusCode::UNAUTHORIZED,
            &APIError::InvalidFlag => StatusCode::BAD_REQUEST,
            &APIError::InvalidPayload => StatusCode::BAD_REQUEST,
            &APIError::Unauthorized => StatusCode::UNAUTHORIZED,
            &APIError::SystemFailure => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl StdError for APIError {
    fn description(&self) -> &str {
        match self {
            APIError::AlreadyExists => "Flag already exists",
            APIError::ConfigFailure => "Server configuration failure",
            APIError::FailedToAccessStore(err) => err.description(),
            APIError::FailedToFind => "Failed to find flag",
            APIError::FailedToParseAuth => "Failed to parse auth payload",
            APIError::FailedToParseBody => "Failed to parse request payload",
            APIError::FailedToParseParams => "Failed to parse request parameters",
            APIError::FailedToSerialize => "Failed to serialize item",
            APIError::FailedToWriteToStore => "Failed to persist to storage",
            APIError::JWTError(err) => err.description(),
            APIError::InvalidFlag => "Provided item is invalid",
            APIError::InvalidPayload => "Provided item is invalid",
            APIError::Unauthorized => "Unauthorized",
            APIError::SystemFailure => "An unknown system failure occured",
        }
    }
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}

impl ResponseError for APIError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(self.status())
    }
}

impl From<SerdeError> for APIError {
    fn from(_: SerdeError) -> APIError {
        APIError::FailedToSerialize
    }
}

impl From<PayloadError> for APIError {
    fn from(_: PayloadError) -> APIError {
        APIError::FailedToParseBody
    }
}

impl From<JsonPayloadError> for APIError {
    fn from(_: JsonPayloadError) -> APIError {
        APIError::FailedToParseBody
    }
}

impl From<JWTError> for APIError {
    fn from(err: JWTError) -> APIError {
        APIError::JWTError(err)
    }
}

impl Into<Box<Future<Item = HttpResponse, Error = APIError>>> for APIError {
    fn into(self) -> Box<Future<Item = HttpResponse, Error = APIError>> {
        Box::new(future::err(self))
    }
}
