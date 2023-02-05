extern crate core;

use core::fmt;
use std::env;
use std::fmt::{Debug, Formatter};

use lambda_http::aws_lambda_events::serde;
use lambda_http::http::StatusCode;
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};
use zana::googlebooks::{Client, VolumeItem};
use zana::ClientError;

#[derive(Debug)]
enum ResponseError {
    LimitExceeded,
    MissingParameter,
    HttpClientError,
    NotFound,
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct FailureResponse {
    error: String,
    details: String,
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
struct SuccessResponse {
    data: BookData,
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
struct BookData {
    page_count: u32,
    ratings_count: u32,
    average_rating: f32,
    description: String,
}

fn fail_response(status: u16, error: ResponseError, details: &str) -> Response<Body> {
    let response = serde_json::to_string(&FailureResponse::new(error, details))
        .expect("could not convert failure response to json");

    Response::builder()
        .header("content-type", "application/json")
        .status(StatusCode::from_u16(status).unwrap_or(StatusCode::SERVICE_UNAVAILABLE))
        .body(Body::Text(response))
        .expect("could not build fail response")
}

fn success_response(item: &VolumeItem) -> Response<Body> {
    let volume_info = &item.info;
    let response = SuccessResponse::new(
        volume_info.page_count,
        volume_info.ratings_count,
        volume_info.average_rating,
        &volume_info.description,
    );

    let response =
        serde_json::to_string(&response).expect("could not convert failure response to json");

    Response::builder()
        .body(Body::Text(response))
        .expect("could not build success response")
}

fn response_from_client_error(error: ClientError) -> Response<Body> {
    match error {
        ClientError::InternalClient(err) => {
            let status_code = err.status().unwrap_or(StatusCode::SERVICE_UNAVAILABLE);
            log::error!("internal http client error {}, {:?}", status_code, err);
            fail_response(
                status_code.as_u16(),
                ResponseError::HttpClientError,
                "Could not retrieve data from external service",
            )
        }
        ClientError::RateLimitExceeded => {
            log::warn!("rate limit exceeded");
            fail_response(
                429,
                ResponseError::LimitExceeded,
                "Rate limit exceeded for external service",
            )
        }
        ClientError::Http(status_code, description) => {
            log::error!(
                "http error while retrieving data {}, {}",
                status_code,
                description
            );
            fail_response(
                status_code,
                ResponseError::HttpClientError,
                "Could not retrieve data",
            )
        }
    }
}

async fn fetch_volume(client: Client, isbn: &str, author: &str, title: &str) -> Response<Body> {
    let volume = match client.volume_by_isbn(isbn).await {
        Ok(volume) => volume,
        Err(err) => return response_from_client_error(err),
    };

    let response = if let Some(items) = volume.items {
        success_response(&items[0])
    } else {
        let volume = match client.volume(&author, &title).await {
            Ok(volume) => volume,
            Err(err) => return response_from_client_error(err),
        };
        if let Some(items) = volume.items {
            success_response(&items[0])
        } else {
            fail_response(404, ResponseError::NotFound, "Volume not found")
        }
    };

    response
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let gbooks_api_key = env::var("GOOGLEBOOKS_API_KEY").expect("API key env variable required");
    let gbooks_api_url = env::var("GOOGLEBOOKS_API_URL").expect("API URL env variable required");

    let client = Client::new(&gbooks_api_key, &gbooks_api_url).expect("to change");

    let isbn = match event.query_string_parameters().first("isbn") {
        Some(isbn) => isbn.to_string(),
        None => {
            return Ok(fail_response(
                400,
                ResponseError::MissingParameter,
                "ISBN is required to retrieve book data",
            ));
        }
    };

    let author = event
        .query_string_parameters()
        .first("author")
        .unwrap_or("")
        .to_string();
    let title = event
        .query_string_parameters()
        .first("title")
        .unwrap_or("")
        .to_string();

    Ok(fetch_volume(client, &isbn, &author, &title).await)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
