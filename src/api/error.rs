use error::BannerError;
use actix_web::{HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use actix_web::error::{JsonPayloadError, PayloadError};
use serde_json::Error as SerdeError;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum APIError {
    AlreadyExists,
    FailedToAccessStore(BannerError),
    FailedToFind,
    FailedToParseAuth,
    FailedToParseBody,
    FailedToParseParams,
    FailedToSerialize,
    FailedToWriteToStore,
    InvalidFlag,
    Unauthorized,
}

impl APIError {
    pub fn status(&self) -> StatusCode {
        match self {
            &APIError::AlreadyExists => StatusCode::CONFLICT,
            &APIError::FailedToAccessStore(_) => StatusCode::INTERNAL_SERVER_ERROR,
            &APIError::FailedToFind => StatusCode::NOT_FOUND,
            &APIError::FailedToParseAuth => StatusCode::BAD_REQUEST,
            &APIError::FailedToParseBody => StatusCode::BAD_REQUEST,
            &APIError::FailedToParseParams => StatusCode::BAD_REQUEST,
            &APIError::FailedToSerialize => StatusCode::INTERNAL_SERVER_ERROR,
            &APIError::FailedToWriteToStore => StatusCode::INTERNAL_SERVER_ERROR,
            &APIError::InvalidFlag => StatusCode::BAD_REQUEST,
            &APIError::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }
}

impl Error for APIError {
    fn description(&self) -> &str {
        match self {
            APIError::AlreadyExists => "Flag already exists",
            APIError::FailedToAccessStore(err) => err.description(),
            APIError::FailedToFind => "Failed to find flag",
            APIError::FailedToParseAuth => "Failed to parse auth payload",
            APIError::FailedToParseBody => "Failed to parse request payload",
            APIError::FailedToParseParams => "Failed to parse request parameters",
            APIError::FailedToSerialize => "Failed to serialize item",
            APIError::FailedToWriteToStore => "Failed to persist to storage",
            APIError::InvalidFlag => "Provided item is invalid",
            APIError::Unauthorized => "Unauthorized",
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