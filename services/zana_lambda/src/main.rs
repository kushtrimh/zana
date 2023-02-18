extern crate core;

mod book;
mod http;

use std::env;

use crate::book::Client;
use crate::http::{failure_response, success_response, RequestType, ResponseError};
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // TODO: Get this from parameter store
    let gbooks_api_key = env::var("GOOGLEBOOKS_API_KEY").expect("API key env variable required");
    let gbooks_api_url = env::var("GOOGLEBOOKS_API_URL").expect("API URL env variable required");

    let request_type: RequestType = match event.query_string_parameters().first("type") {
        Some(request_type) => match request_type.parse() {
            Ok(request_type) => request_type,
            Err(_) => {
                return failure_response(ResponseError::MissingParameter(String::from(
                    "Invalid type",
                )))
            }
        },
        None => {
            return failure_response(ResponseError::MissingParameter(String::from(
                "Type is required",
            )))
        }
    };

    let isbn = event
        .query_string_parameters()
        .first("isbn")
        .unwrap_or("")
        .to_string();
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

    let client = Client::new(("", ""), "");

    match client {
        Ok(client) => {
            let query = if isbn != "" {
                log::debug!("fetching book by isbn {} for {:?}", isbn, &request_type);
                client.fetch_by_isbn(&request_type, &isbn).await
            } else if author != "" && title != "" {
                log::debug!(
                    "fetching book by title {} and author {} for {:?}",
                    title,
                    author,
                    &request_type
                );
                client
                    .fetch_by_title_and_author(&request_type, &title, &author)
                    .await
            } else {
                return failure_response(ResponseError::MissingParameter(String::from(
                    "Either ISBN or title and author must be provided",
                )));
            };
            match query {
                Ok(book) => success_response(&book),
                Err(err) => {
                    log::error!("could not fetch book for {:?}, {:?}", &request_type, err);
                    failure_response(err)
                }
            }
        }
        Err(err) => {
            log::error!(
                "could not create client for type {:?}, {:?}",
                &request_type,
                err
            );
            failure_response(ResponseError::ServiceError)
        }
    }
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
