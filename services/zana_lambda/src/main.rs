extern crate core;

mod book;

use std::env;

use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use zana::googlebooks::Client;

use crate::book::{fail_response, fetch_volume, ResponseError};

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let gbooks_api_key = env::var("GOOGLEBOOKS_API_KEY").expect("API key env variable required");
    let gbooks_api_url = env::var("GOOGLEBOOKS_API_URL").expect("API URL env variable required");

    let client = Client::new(&gbooks_api_key, &gbooks_api_url)?;

    let isbn = match event.query_string_parameters().first("isbn") {
        Some(isbn) => isbn.to_string(),
        None => {
            return fail_response(
                400,
                ResponseError::MissingParameter,
                "ISBN is required to retrieve book data",
            );
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

    match fetch_volume(client, &isbn, &author, &title).await {
        Ok(response) => Ok(response),
        Err(err) => {
            log::error!("could not retrieve data {:?}", err);
            fail_response(503, ResponseError::ServiceError, "Could not retrieve data")
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
