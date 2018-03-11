use iron::status::Status;

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
}

impl APIError {
    pub fn status(&self) -> Status {
        match self {
            &APIError::AlreadyExists => Status::Conflict,
            &APIError::FailedToAccessParams => Status::BadRequest,
            &APIError::FailedToAccessStore => Status::InternalServerError,
            &APIError::FailedToFind => Status::NotFound,
            &APIError::FailedToParseBody => Status::BadRequest,
            &APIError::FailedToParseParams => Status::BadRequest,
            &APIError::FailedToSerialize => Status::InternalServerError,
        }
    }
}

impl Error for APIError {
    fn description(&self) -> &str {
        self.status().canonical_reason().unwrap_or("")
    }
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}
