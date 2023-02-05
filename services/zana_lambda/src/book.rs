use core::fmt;
use std::fmt::{Debug, Formatter};

use lambda_http::http::StatusCode;
use lambda_http::{Body, Error, Response};
use serde::{Deserialize, Serialize};
use zana::googlebooks::{Client, VolumeItem};
use zana::ClientError;

#[derive(Debug)]
pub enum ResponseError {
    LimitExceeded,
    MissingParameter,
    HttpClientError,
    NotFound,
    ServiceError,
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
    fn new(error: ResponseError, details: &str) -> Self {
        FailureResponse {
            error: String::from(error.to_string()),
            details: String::from(details),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct SuccessResponse {
    pub data: BookData,
}

impl SuccessResponse {
    fn new(
        page_count: u32,
        ratings_count: u32,
        average_rating: f32,
        description: &str,
    ) -> SuccessResponse {
        SuccessResponse {
            data: BookData {
                page_count,
                ratings_count,
                average_rating,
                description: String::from(description),
            },
        }
    }
}

#[derive(Serialize, Debug)]
pub struct BookData {
    pub page_count: u32,
    pub ratings_count: u32,
    pub average_rating: f32,
    pub description: String,
}

pub fn fail_response(
    status: u16,
    error: ResponseError,
    details: &str,
) -> Result<Response<Body>, Error> {
    let response = serde_json::to_string(&FailureResponse::new(error, details))?;

    Ok(Response::builder()
        .header("content-type", "application/json")
        .status(StatusCode::from_u16(status).unwrap_or(StatusCode::SERVICE_UNAVAILABLE))
        .body(Body::Text(response))?)
}

fn success_response(item: &VolumeItem) -> Result<Response<Body>, Error> {
    let volume_info = &item.info;
    let response = SuccessResponse::new(
        volume_info.page_count,
        volume_info.ratings_count,
        volume_info.average_rating,
        &volume_info.description,
    );

    let response = serde_json::to_string(&response)?;

    Ok(Response::builder()
        .header("content-type", "application/json")
        .status(200)
        .body(Body::Text(response))?)
}

fn response_from_client_error(error: ClientError) -> Result<Response<Body>, Error> {
    let (status_code, error_type, details) = match error {
        ClientError::InternalClient(err) => {
            let status_code = err.status().unwrap_or(StatusCode::SERVICE_UNAVAILABLE);
            log::error!("internal http client error {}, {:?}", status_code, err);
            (
                status_code.as_u16(),
                ResponseError::HttpClientError,
                "Could not retrieve data from external service",
            )
        }
        ClientError::RateLimitExceeded => {
            log::warn!("rate limit exceeded");
            (
                429,
                ResponseError::LimitExceeded,
                "Rate limit exceeded for external service",
            )
        },
        ClientError::Http(status_code, description) => {
            log::error!(
                "http error while retrieving data {}, {}",
                status_code,
                description
            );
            (
                status_code,
                ResponseError::HttpClientError,
                "Could not retrieve data",
            )
        },
    };
    fail_response(status_code, error_type, details)
}

pub async fn fetch_volume(
    client: Client,
    isbn: &str,
    author: &str,
    title: &str,
) -> Result<Response<Body>, Error> {
    log::debug!("sending volume query request for isbn: {}", isbn);
    let volume = match client.volume_by_isbn(isbn).await {
        Ok(volume) => volume,
        Err(err) => return response_from_client_error(err),
    };

    let response = if let Some(items) = volume.items {
        success_response(&items[0])
    } else {
        log::debug!("volume query request for isbn {} returned no data", isbn);
        log::debug!(
            "sending volume query request for title {} of author {}",
            title,
            author
        );
        let volume = match client.volume(&author, &title).await {
            Ok(volume) => volume,
            Err(err) => return response_from_client_error(err),
        };
        if let Some(items) = volume.items {
            success_response(&items[0])
        } else {
            log::debug!(
                "volume query request for title {} of author {} returned no data",
                title,
                author
            );
            fail_response(404, ResponseError::NotFound, "Volume not found")
        }
    };

    response
}

#[cfg(test)]
mod tests {

    #[test]
    fn create_success_response() {}

    #[test]
    fn create_failure_response() {}

    #[test]
    fn create_response_from_client_error() {}
}
