extern crate core;

use std::env;

use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use zana::{googlebooks, openlibrary};

use crate::book::Client;
use crate::http::{failure_response, success_response, RequestType, ResponseError};
use crate::params::{AWSParamStore, ParamStore};

mod book;
mod http;
mod params;

// TODO: test this with httpmock, we would need to mock http requests and

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let parameter_store_port = env::var("PARAMETERS_SECRETS_EXTENSION_HTTP_PORT")
        .expect("environment variable 'PARAMETERS_SECRETS_EXTENSION_HTTP_PORT' not set");
    let aws_token =
        env::var("AWS_SESSION_TOKEN").expect("environment variable 'AWS_SESSION_TOKEN' not set");
    let zana_env = env::var("ZANA_ENV").expect("environment variable 'ZANA_ENV' not set");
    let parameter_store_url = format!(
        "http://localhost:{}/systemsmanager/parameters/get",
        parameter_store_port
    );

    let request_type: RequestType = match event.query_string_parameters().first("type") {
        Some(request_type) => match request_type.parse() {
            Ok(request_type) => request_type,
            Err(_) => {
                return failure_response(ResponseError::MissingParameter(String::from(
                    "Invalid type",
                )));
            }
        },
        None => {
            return failure_response(ResponseError::MissingParameter(String::from(
                "Type is required",
            )));
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

    let param_store = AWSParamStore::new(&parameter_store_url, &aws_token, &zana_env);
    let googlebooks_url: String = param_store
        .parameter("/zana/google-books-url", false)
        .await?;
    let googlebooks_key: String = param_store
        .parameter("/zana/google-books-key", true)
        .await?;
    let openlibrary_url: String = param_store
        .parameter("/zana/openlibrary-url", false)
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

    let query = if !isbn.is_empty() {
        log::debug!("fetching book by isbn {} for {:?}", isbn, &request_type);
        client.fetch_by_isbn(&request_type, &isbn).await
    } else if !author.is_empty() && !title.is_empty() {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use lambda_http::aws_lambda_events::query_map::QueryMap;
    use lambda_http::http::StatusCode;
    use lambda_http::{Request, RequestExt};

    use crate::function_handler;

    #[tokio::test]
    async fn sample_test() {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("type".to_string(), "test".to_string());
        let request = Request::default().with_query_string_parameters(QueryMap::from(params));

        match function_handler(request).await {
            Ok(response) => assert_eq!(response.status(), StatusCode::BAD_REQUEST),
            Err(err) => panic!("error {:?}", err),
        };
    }
}
