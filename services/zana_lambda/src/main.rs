extern crate core;

use std::env;

use lambda_http::{run, service_fn, Body, Error, Request, Response};
use zana::{googlebooks, openlibrary};

use zana_lambda::book::Client;
use zana_lambda::http;
use zana_lambda::http::{failure_response, success_response, RequestType, ResponseError};
use zana_lambda::params::{AWSParamStore, ParamStore};

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Required env variables
    let zana_env = env::var("ZANA_ENV").expect("environment variable 'ZANA_ENV' not set");

    // Env variables set by AWS
    let parameter_store_port = env::var("PARAMETERS_SECRETS_EXTENSION_HTTP_PORT")
        .expect("environment variable 'PARAMETERS_SECRETS_EXTENSION_HTTP_PORT' not set");
    let aws_token =
        env::var("AWS_SESSION_TOKEN").expect("environment variable 'AWS_SESSION_TOKEN' not set");
    let parameter_store_url = format!(
        "http://localhost:{}/systemsmanager/parameters/get",
        parameter_store_port
    );

    let request_type: RequestType = match http::request_type(&event) {
        Ok(request_type) => request_type,
        Err(err) => return failure_response(err),
    };
    let isbn = http::query_parameter(&event, "isbn", "");
    let author = http::query_parameter(&event, "author", "");
    let title = http::query_parameter(&event, "title", "");

    let param_store = AWSParamStore::new(&parameter_store_url, &aws_token, &zana_env);

    let googlebooks_url = param_store
        .parameter_from_env("ZANA_GOOGLE_BOOKS_URL", "/zana/google-books-url", false)
        .await?;
    let googlebooks_key: String = param_store
        .parameter_from_env("ZANA_GOOGLE_BOOKS_KEY", "/zana/google-books-key", true)
        .await?;
    let openlibrary_url: String = param_store
        .parameter_from_env("ZANA_OPENLIBRARY_URL", "/zana/openlibrary-url", false)
        .await?;

    let googlebooks_client = match googlebooks::Client::new(&googlebooks_key, &googlebooks_url) {
        Ok(client) => Box::new(client),
        Err(err) => return failure_response(ResponseError::BookClientError(err)),
    };

    let openlibrary_client = match openlibrary::Client::new(&openlibrary_url) {
        Ok(client) => Box::new(client),
        Err(err) => return failure_response(ResponseError::BookClientError(err)),
    };

    let client = Client::new(googlebooks_client, openlibrary_client);

    let book = client
        .fetch_book(&request_type, &isbn, &title, &author)
        .await;
    match book {
        Ok(book) => success_response(&book),
        Err(err) => {
            tracing::error!("could not fetch book for {:?}, {:?}", &request_type, err);
            failure_response(err)
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
