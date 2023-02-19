use core::fmt;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;

use lambda_http::http::StatusCode;
use lambda_http::{Body, Error, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zana::{Book, ClientError};

// TODO: unit tests only here, no external calls made

#[derive(Debug, PartialEq)]
pub enum RequestType {
    GoogleBooks,
    OpenLibrary,
}

impl FromStr for RequestType {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "googlebooks" => Ok(Self::GoogleBooks),
            "openlibrary" => Ok(Self::OpenLibrary),
            _ => Err(()),
        }
    }
}

#[derive(Error, Debug)]
pub enum ResponseError {
    MissingParameter(String),
    BookClientError(#[from] ClientError),
    HttpClientError(#[from] reqwest::Error),
    ServiceError,
}

impl ResponseError {
    fn status_and_details(&self) -> (u16, &str) {
        match self {
            ResponseError::MissingParameter(details) => (400, details),
            ResponseError::BookClientError(err) => {
                let status_and_details = match err {
                    ClientError::InternalClient(_) => {
                        (503, "Could not retrieve data from external service")
                    }
                    ClientError::RateLimitExceeded => {
                        (429, "Rate limit exceeded for external service")
                    }
                    ClientError::NotFound => (404, "Book not found"),
                    ClientError::Http(status_code, details) => (*status_code, details.as_str()),
                };
                log::error!(
                    "book client error {}/{}: {}",
                    status_and_details.0,
                    status_and_details.1,
                    err
                );
                status_and_details
            }
            ResponseError::HttpClientError(err) => {
                let status_and_details = match err.status() {
                    Some(status_code) => (
                        status_code.as_u16(),
                        "Something went wrong, please try again.",
                    ),
                    None => (500, "Something went wrong, please try again."),
                };
                log::error!(
                    "http client error {}/{}: {}",
                    status_and_details.0,
                    status_and_details.1,
                    err
                );
                status_and_details
            }
            _ => (500, "Something went wrong, please try again."),
        }
    }
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FailureResponse {
    pub error: String,
    pub details: String,
}

impl fmt::Display for FailureResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.error, self.details)
    }
}

impl std::error::Error for FailureResponse {}

impl FailureResponse {
    pub fn new_with_details(error_type: &ResponseError, details: &str) -> Self {
        FailureResponse {
            error: error_type.to_string(),
            details: String::from(details),
        }
    }

    pub fn new(error_type: &ResponseError) -> Self {
        let (_, details) = error_type.status_and_details();
        FailureResponse {
            error: error_type.to_string(),
            details: String::from(details),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct SuccessResponse {
    pub data: BookData,
    pub rating: Option<RatingData>,
}

impl SuccessResponse {
    fn new(data: BookData) -> Self {
        SuccessResponse { data, rating: None }
    }
}

#[derive(Serialize, Debug)]
pub struct BookData {
    pub page_count: u32,
    pub description: String,
}

impl BookData {
    pub fn new(page_count: u32, description: &str) -> Self {
        Self {
            page_count,
            description: String::from(description),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct RatingData {
    pub average_rating: f32,
    pub ratings_count: u32,
}

impl RatingData {
    pub fn new(average_rating: f32, ratings_count: u32) -> Self {
        Self {
            average_rating,
            ratings_count,
        }
    }
}

pub fn failure_response(error: ResponseError) -> Result<Response<Body>, Error> {
    let (status, details) = error.status_and_details();
    let response = serde_json::to_string(&FailureResponse::new_with_details(&error, details))?;

    Ok(Response::builder()
        .header("content-type", "application/json")
        .status(StatusCode::from_u16(status).unwrap_or(StatusCode::SERVICE_UNAVAILABLE))
        .body(Body::Text(response))?)
}

pub fn success_response(book: &Book) -> Result<Response<Body>, Error> {
    let mut response = SuccessResponse::new(BookData::new(book.page_count, &book.description));

    if let Some(rating) = &book.rating {
        response.rating = Some(RatingData::new(rating.average_rating, rating.ratings_count));
    }
    let response = serde_json::to_string(&response)?;

    Ok(Response::builder()
        .header("content-type", "application/json")
        .status(200)
        .body(Body::Text(response))?)
}
