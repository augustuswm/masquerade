use actix_web::{Body, HttpResponse, ResponseError, StatusCode};
use actix_web::error::PayloadError;

use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum APIError {
    AlreadyExists,
    FailedToAccessParams,
    FailedToAccessStore,
    FailedToFind,
    FailedToParseBody,
    FailedToParseParams,
    FailedToSerialize,
    FailedToWriteToStore,
}

impl APIError {
    pub fn status(&self) -> StatusCode {
        match self {
            &APIError::AlreadyExists => StatusCode::CONFLICT,
            &APIError::FailedToAccessParams => StatusCode::BAD_REQUEST,
            &APIError::FailedToAccessStore => StatusCode::INTERNAL_SERVER_ERROR,
            &APIError::FailedToFind => StatusCode::NOT_FOUND,
            &APIError::FailedToParseBody => StatusCode::BAD_REQUEST,
            &APIError::FailedToParseParams => StatusCode::BAD_REQUEST,
            &APIError::FailedToSerialize => StatusCode::INTERNAL_SERVER_ERROR,
            &APIError::FailedToWriteToStore => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl Error for APIError {
    fn description(&self) -> &str {
        ""
    }
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}

impl ResponseError for APIError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(self.status(), Body::Empty)
    }
}

impl From<PayloadError> for APIError {
    fn from(err: PayloadError) -> APIError {
        APIError::FailedToParseBody
    }
}
