use core::fmt;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;

use lambda_http::http::StatusCode;
use lambda_http::{Body, Error, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zana::{Book, ClientError};

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
    HttpClientError(#[from] ClientError),
}

impl ResponseError {
    fn status(&self) -> u16 {
        match self {
            ResponseError::MissingParameter(_) => 400,
            ResponseError::HttpClientError(err) => match err {
                ClientError::InternalClient(_) => 503,
                ClientError::RateLimitExceeded => 429,
                ClientError::NotFound => 404,
                ClientError::Http(status_code, _) => *status_code,
            },
        }
    }

    fn details(&self) -> &str {
        match self {
            ResponseError::MissingParameter(details) => details,
            ResponseError::HttpClientError(err) => match err {
                ClientError::InternalClient(_) => "Could not retrieve data from external service",
                ClientError::RateLimitExceeded => "Rate limit exceeded for external service",
                ClientError::NotFound => "Book not found",
                ClientError::Http(_, details) => details,
            },
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

impl FailureResponse {
    fn new(error: &ResponseError) -> Self {
        FailureResponse {
            error: String::from(error.to_string()),
            details: String::from(error.details()),
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
    let response = serde_json::to_string(&FailureResponse::new(&error))?;

    Ok(Response::builder()
        .header("content-type", "application/json")
        .status(StatusCode::from_u16(error.status()).unwrap_or(StatusCode::SERVICE_UNAVAILABLE))
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
