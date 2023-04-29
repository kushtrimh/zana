/*!
Provides functions and types to manage HTTP requests, request parameters, and responses.

This module provides functions that create successful and failed responses, retrieve parameters from requests,
and deal with HTTP error handling.
 */
use core::fmt;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;

use lambda_http::http::StatusCode;
use lambda_http::{Body, Error, RequestExt, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zana::{Book, ClientError};

/// Enum that represents all supported book data providers.
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

/// An error that occurs during request handling.
///
/// The error contains different variants based on the source of the error.
#[derive(Error, Debug)]
pub enum ResponseError {
    /// Occurs when a request parameter is missing or is invalid.
    MissingParameter(String),
    /// Occurs when an error is returned from a request made from [`zana`](zana) clients.
    BookClientError(#[from] ClientError),
    /// Occurs for any error that comes from [`reqwest`](reqwest) crate. This will include errors
    /// when a timeout is reached, or no connection can be made to the specified endpoint.
    HttpClientError(#[from] reqwest::Error),
    /// Generic error that occurs during request handling.
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
                status_and_details
            }
            _ => (500, "Something went wrong, please try again."),
        }
    }
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let error_name = match self {
            ResponseError::MissingParameter(_) => "MissingParameter",
            ResponseError::BookClientError(client_error) => match client_error {
                ClientError::RateLimitExceeded => "RateLimitExceeded",
                ClientError::NotFound => "NotFound",
                ClientError::Http(_, _) | ClientError::InternalClient(_) => "HttpClientError",
            },
            ResponseError::HttpClientError(_) => "HttpClientError",
            ResponseError::ServiceError => "ServiceError",
        };
        write!(f, "{}", error_name)
    }
}

/// Response used to represent an error during request handling.
///
/// Errors include HTTP client errors during requests, missing or invalid parameter,
/// a book not being found, and any error during data handling of requests and responses.
#[derive(Serialize, Deserialize, Debug)]
pub struct FailureResponse {
    pub error: String,
    pub details: String,
    pub status_code: u16,
}

impl fmt::Display for FailureResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.error, self.details)
    }
}

impl std::error::Error for FailureResponse {}

impl FailureResponse {
    /// Returns new response from a specific type of error, and specific details and status code.
    pub fn new_with_details(error_type: &ResponseError, details: &str, status_code: u16) -> Self {
        FailureResponse {
            error: error_type.to_string(),
            details: String::from(details),
            status_code,
        }
    }

    /// Returns new response from a specific type of error.
    ///
    /// Status code and details that are set on the response are retrieved from the error type.
    pub fn new(error_type: &ResponseError) -> Self {
        let (status_code, details) = error_type.status_and_details();
        FailureResponse {
            error: error_type.to_string(),
            details: String::from(details),
            status_code,
        }
    }
}

/// Response used to represent a retrieved book.
///
/// Ratings are by default not required, and set to `None`, since not all providers may support them,
/// and not all books will have ratings attached when retrieved from providers.
#[derive(Serialize, Deserialize, Debug)]
pub struct SuccessResponse {
    pub data: BookData,
    pub rating: Option<RatingData>,
}

impl SuccessResponse {
    fn new(data: BookData) -> Self {
        SuccessResponse { data, rating: None }
    }
}

/// Represents a book and some of its data.
#[derive(Serialize, Deserialize, Debug)]
pub struct BookData {
    pub page_count: u32,
    pub description: String,
    pub provider_link: String,
}

impl BookData {
    pub fn new(page_count: u32, description: &str, provider_link: &str) -> Self {
        Self {
            page_count,
            description: String::from(description),
            provider_link: String::from(provider_link),
        }
    }
}

/// Represents a ratings about a specific book.
#[derive(Serialize, Deserialize, Debug)]
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

/// Returns a new failure response or an error if the response could not be constructed.
///
/// Response is returned as JSON and content type is set to `application/json` by default.
/// `503` status code is provided by errors if no other specific status code is available from the error.
pub fn failure_response(error: ResponseError) -> Result<Response<Body>, Error> {
    let (status, details) = error.status_and_details();
    let response =
        serde_json::to_string(&FailureResponse::new_with_details(&error, details, status))?;

    Ok(Response::builder()
        .header("content-type", "application/json")
        .status(StatusCode::from_u16(status).unwrap_or(StatusCode::SERVICE_UNAVAILABLE))
        .body(Body::Text(response))?)
}

/// Returns a new success response or an error if the response could not be constructed.
///
/// Response is returned as JSON and content type is set to `application/json` by default.
pub fn success_response(book: &Book) -> Result<Response<Body>, Error> {
    let mut response = SuccessResponse::new(BookData::new(
        book.page_count,
        &book.description,
        &book.provider_link,
    ));

    if let Some(rating) = &book.rating {
        response.rating = Some(RatingData::new(rating.average_rating, rating.ratings_count));
    }
    let response = serde_json::to_string(&response)?;

    Ok(Response::builder()
        .header("content-type", "application/json")
        .status(200)
        .body(Body::Text(response))?)
}

/// Returns a query parameter from the request, or the default provided value if the parameter
/// is missing.
pub fn query_parameter(request: &impl RequestExt, name: &str, default: &str) -> String {
    request
        .query_string_parameters()
        .first(name)
        .unwrap_or(default)
        .to_string()
}

/// Returns [`RequestType`](enum@RequestType) from the `type` query parameter.
///
/// If the `type` query parameter is missing or is invalid, an error is returned instead.
pub fn request_type(request: &impl RequestExt) -> Result<RequestType, ResponseError> {
    let request_type: RequestType = match request.query_string_parameters().first("type") {
        Some(request_type) => match request_type.parse() {
            Ok(request_type) => request_type,
            Err(_) => {
                return Err(ResponseError::MissingParameter(String::from(
                    "Invalid type",
                )));
            }
        },
        None => {
            return Err(ResponseError::MissingParameter(String::from(
                "Type is required",
            )));
        }
    };
    Ok(request_type)
}

#[cfg(test)]
mod tests {
    use crate::http::{
        failure_response, query_parameter, request_type, success_response, FailureResponse,
        RequestType, ResponseError, SuccessResponse,
    };
    use lambda_http::aws_lambda_events::query_map::QueryMap;
    use lambda_http::ext::PayloadError;
    use lambda_http::request::RequestContext;
    use lambda_http::RequestExt;
    use lambda_runtime::Context;
    use reqwest::StatusCode;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::str::FromStr;
    use zana::{Book, ClientError, Rating};

    struct TestRequest {
        query_map: QueryMap,
    }

    impl TestRequest {
        fn new(query_params: HashMap<String, String>) -> Self {
            Self {
                query_map: query_params.into(),
            }
        }
    }

    impl RequestExt for TestRequest {
        fn raw_http_path(&self) -> String {
            unimplemented!()
        }

        fn with_raw_http_path(self, _path: &str) -> Self {
            unimplemented!()
        }

        fn query_string_parameters(&self) -> QueryMap {
            self.query_map.clone()
        }

        fn with_query_string_parameters<Q>(self, _parameters: Q) -> Self
        where
            Q: Into<QueryMap>,
        {
            unimplemented!()
        }

        fn path_parameters(&self) -> QueryMap {
            unimplemented!()
        }

        fn with_path_parameters<P>(self, _parameters: P) -> Self
        where
            P: Into<QueryMap>,
        {
            unimplemented!()
        }

        fn stage_variables(&self) -> QueryMap {
            unimplemented!()
        }

        fn request_context(&self) -> RequestContext {
            unimplemented!()
        }

        fn with_request_context(self, _context: RequestContext) -> Self {
            unimplemented!()
        }

        fn payload<D>(&self) -> Result<Option<D>, PayloadError>
        where
            for<'de> D: Deserialize<'de>,
        {
            unimplemented!()
        }

        fn lambda_context(&self) -> Context {
            unimplemented!()
        }

        fn with_lambda_context(self, _context: Context) -> Self {
            unimplemented!()
        }
    }

    fn assert_request_type_err(request: &impl RequestExt, expected_message: &str) {
        match request_type(request) {
            Ok(_) => panic!("request type not expected when not provided as param"),
            Err(err) => match err {
                ResponseError::MissingParameter(message) => assert_eq!(expected_message, message),
                _ => panic!("invalid error returned"),
            },
        }
    }

    fn assert_book_success_response(book: &Book) {
        let response = success_response(&book).expect("response expected to be present");
        let body = String::from_utf8(response.body().to_vec()).expect("utf8 string expected");
        let response_book: SuccessResponse =
            serde_json::from_str(&body).expect("response expected to be parsed");

        assert_eq!(StatusCode::OK, response.status());
        assert_book_equality(book, response_book);
    }

    fn assert_book_equality(book: &Book, response_book: SuccessResponse) {
        assert_eq!(book.page_count, response_book.data.page_count);
        assert_eq!(book.provider_link, response_book.data.provider_link);
        assert_eq!(book.description, response_book.data.description);

        if let Some(book_rating) = &book.rating {
            let response_book_rating = response_book.rating.expect("rating expected");
            assert_eq!(
                book_rating.ratings_count,
                response_book_rating.ratings_count
            );
            assert_eq!(
                book_rating.average_rating,
                response_book_rating.average_rating
            );
        }
    }

    #[test]
    fn gb_request_type_from_query_param() {
        let request_type = RequestType::from_str("googlebooks")
            .expect("expected to parse query value to request type");
        assert_eq!(request_type, RequestType::GoogleBooks);
    }

    #[test]
    fn ol_request_type_from_query_param() {
        let request_type = RequestType::from_str("openlibrary")
            .expect("expected to parse query value to request type");
        assert_eq!(request_type, RequestType::OpenLibrary);
    }

    #[test]
    fn status_code_400_on_missing_parameter() {
        assert_eq!(
            400,
            ResponseError::MissingParameter(String::from("param missing"))
                .status_and_details()
                .0
        );
    }

    #[test]
    fn status_code_404_on_missing_book() {
        assert_eq!(
            404,
            ResponseError::BookClientError(ClientError::NotFound)
                .status_and_details()
                .0
        );
    }

    #[test]
    fn status_code_429_on_exceeded_limit() {
        assert_eq!(
            429,
            ResponseError::BookClientError(ClientError::RateLimitExceeded)
                .status_and_details()
                .0
        );
    }

    #[test]
    fn status_code_from_book_client_http_error() {
        let status_code: u16 = 500;
        assert_eq!(
            status_code,
            ResponseError::BookClientError(ClientError::Http(status_code, String::new()))
                .status_and_details()
                .0
        );
    }

    #[test]
    fn response_from_response_error() {
        let error_message = String::from("ISBN parameter missing");
        let response_error = ResponseError::MissingParameter(error_message.clone());
        let expected_error = FailureResponse::new(&response_error);

        let response = failure_response(response_error).expect("response expected to be present");
        let body = String::from_utf8(response.body().to_vec()).expect("utf8 string expected");
        let expected_response =
            serde_json::to_string(&expected_error).expect("could not convert to json");

        assert_eq!(StatusCode::BAD_REQUEST, response.status());
        assert_eq!(expected_response, body);
    }

    #[test]
    fn response_from_book() {
        let rating = Rating::new(4.5, 123);
        let book = Book::new_with_rating(
            531,
            "Book description here",
            "http://localhost/link/to/book",
            rating,
        );
        assert_book_success_response(&book);
    }

    #[test]
    fn response_from_book_without_ratings() {
        let book = Book::new(
            531,
            "Book description here",
            "http://localhost/link/to/book",
        );
        assert_book_success_response(&book);
    }

    #[test]
    fn query_parameter_when_it_exists() {
        let param = "param-name";
        let value = "param-value";
        let params = HashMap::from([(String::from(param), String::from(value))]);
        let request = TestRequest::new(params);
        assert_eq!(value, query_parameter(&request, param, ""));
    }

    #[test]
    fn query_parameter_when_its_missing() {
        let param = "param-name";
        let default = "default";
        let request = TestRequest::new(HashMap::new());
        assert_eq!(default, query_parameter(&request, param, default));
    }

    #[test]
    fn request_type_provided() {
        let request = TestRequest::new(HashMap::from([(
            String::from("type"),
            String::from("googlebooks"),
        )]));
        let request_type = request_type(&request).expect("could not retrieve type");
        assert_eq!(RequestType::GoogleBooks, request_type);
    }

    #[test]
    fn request_type_invalid() {
        let request = TestRequest::new(HashMap::from([(
            String::from("type"),
            String::from("invalid"),
        )]));
        assert_request_type_err(&request, "Invalid type");
    }

    #[test]
    fn request_type_missing() {
        let request = TestRequest::new(HashMap::new());
        assert_request_type_err(&request, "Type is required");
    }
}
