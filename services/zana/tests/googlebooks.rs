mod util;

use httpmock::prelude::*;
use httpmock::Mock;

use crate::util::{get_sample, set_property_to_null};
use zana::googlebooks::Client;
use zana::{Book, BookClient, ClientError};

const API_KEY: &str = "b85a45ddd5a99124cf4ec9a74f93fcf1";
const VOLUME_PATH: &str = "/books/v1/volumes";

fn create_client(server: &MockServer) -> Client {
    Client::new(API_KEY, &format!("http://{}", &server.address())).expect("could not create client")
}

fn assert_book_equality(book: Book) {
    let rating = book.rating.expect("ratings should exist");

    assert_eq!(560, book.page_count);
    assert_eq!(book.description, "The first novel in the First Law Trilogy",);
    assert_eq!(3.5, rating.average_rating);
    assert_eq!(107, rating.ratings_count);
}

async fn assert_response(
    isbn: &str,
    status_code: u16,
    response: &str,
) -> Result<Book, ClientError> {
    let server = MockServer::start();
    let m = create_mock(&server, &format!("isbn:{}", isbn), status_code, &response);

    let client = create_client(&server);
    let book = client.book_by_isbn(isbn).await;
    m.assert();
    book
}

fn create_mock<'a>(
    server: &'a MockServer,
    query: &str,
    status_code: u16,
    response_body: &str,
) -> Mock<'a> {
    server.mock(|when, then| {
        when.method(GET)
            .path(VOLUME_PATH)
            .query_param("key", API_KEY)
            .query_param("q", query)
            .query_param("maxResults", "1")
            .query_param("fields", "items")
            .header("Accept-Encoding", "gzip");
        then.status(status_code)
            .header("Content-Type", "application/json")
            .body(response_body);
    })
}

#[tokio::test]
async fn fetch_book_by_isbn() {
    let isbn = "9780316387316";

    let book = assert_response(isbn, 200, &get_sample("googlebooks_volume.json"))
        .await
        .expect("could not get book by isbn");
    assert_book_equality(book);
}

#[tokio::test]
async fn fetch_book_by_name_and_author() {
    let author = "Joe Abercrombie";
    let title = "The Blade Itself";

    let server = MockServer::start();
    let m = create_mock(
        &server,
        &format!("inauthor:{} intitle:{}", author, title),
        200,
        &get_sample("googlebooks_volume.json"),
    );

    let client = create_client(&server);
    let book = client
        .book(author, title)
        .await
        .expect("could not get book by title and author");

    m.assert();
    assert_book_equality(book);
}

#[tokio::test]
async fn handle_empty_book_response() {
    let isbn = "9780316387316";

    let book = assert_response(isbn, 200, "{}").await;
    let _returned_error = book
        .err()
        .expect("error not returned when expected for missing book");
    assert!(matches!(ClientError::NotFound, _returned_error));
}

#[tokio::test]
async fn return_rate_limit_error() {
    let isbn = "9780316387316";

    for status_code in [429, 403] {
        let book = assert_response(isbn, status_code, "").await;
        let _returned_error = book.err().expect(&format!(
            "error not returned when expected for status {}",
            status_code
        ));
        assert!(matches!(ClientError::RateLimitExceeded, _returned_error));
    }
}

#[tokio::test]
async fn handle_response_with_null_average_rating() {
    let isbn = "9780316387316";
    let json_value = set_property_to_null(
        "googlebooks_volume.json",
        "/items/0/volumeInfo/averageRating",
    );
    let response = json_value.to_string();
    let book = assert_response(isbn, 200, &response)
        .await
        .expect("could not get book by isbn");
    assert!(book.rating.is_none());
}

#[tokio::test]
async fn handle_response_with_null_ratings_count() {
    let isbn = "9780316387316";
    let json_value = set_property_to_null(
        "googlebooks_volume.json",
        "/items/0/volumeInfo/ratingsCount",
    );
    let response = json_value.to_string();
    let book = assert_response(isbn, 200, &response)
        .await
        .expect("could not get book by isbn");
    assert!(book.rating.is_none());
}

#[tokio::test]
async fn handle_response_with_null_description() {
    let isbn = "9780316387316";
    let json_value =
        set_property_to_null("googlebooks_volume.json", "/items/0/volumeInfo/description");
    let response = json_value.to_string();
    let book = assert_response(isbn, 200, &response)
        .await
        .expect("could not get book by isbn");

    assert!(book.description.is_empty());
}

#[tokio::test]
async fn handle_response_with_null_page_count() {
    let isbn = "9780316387316";
    let json_value =
        set_property_to_null("googlebooks_volume.json", "/items/0/volumeInfo/pageCount");
    let response = json_value.to_string();
    let book = assert_response(isbn, 200, &response)
        .await
        .expect("could not get book by isbn");

    assert_eq!(0, book.page_count);
}

#[tokio::test]
async fn handle_other_http_error() {
    let isbn = "9780316387316";

    let server = MockServer::start();
    let expected_status_code = 400;
    let m = create_mock(
        &server,
        &format!("isbn:{}", isbn),
        expected_status_code,
        "{\"error\": {}}",
    );

    let client = create_client(&server);
    let book = client.book_by_isbn(isbn).await;

    m.assert();
    let returned_error = book.err().expect("error not returned when expected");
    match returned_error {
        ClientError::Http(status_code, _) => {
            assert_eq!(expected_status_code, status_code);
        }
        _ => {
            panic!("invalid error type returned")
        }
    }
}
