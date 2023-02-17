mod util;

use httpmock::prelude::*;
use httpmock::Mock;
use zana::{Book, BookClient, ClientError, Rating};

use crate::util::get_sample;
use zana::openlibrary::Client;

const ISBN_PATH: &str = "/isbn";
const RATINGS_PATH: &str = "/ratings";
const WORKS_PATH: &str = "/works/OL8400950W";

fn create_client(server: &MockServer) -> Client {
    Client::new(&format!("http://{}", &server.address())).expect("could not create client")
}

fn create_mock<'a>(
    server: &'a MockServer,
    path: &str,
    status_code: u16,
    response_body: &str,
) -> Mock<'a> {
    server.mock(|when, then| {
        when.method(GET).path(path);
        then.status(status_code)
            .header("Content-Type", "application/json")
            .body(response_body);
    })
}

fn create_default_expected_book() -> Book {
    let ratings = Rating::new(4.5, 23);
    let description = "Logen Ninefingers, infamous barbarian, has finally run out of luck.";
    Book::new_with_rating(542, description, ratings)
}

async fn assert_successful_fetch(
    isbn: &str,
    isbn_sample: &str,
    works_sample: &str,
    ratings_sample: &str,
) -> Book {
    let server = MockServer::start();
    let isbn_mock = create_mock(
        &server,
        &format!("{}/{}.json", ISBN_PATH, isbn),
        200,
        isbn_sample,
    );

    let works_mock = create_mock(&server, &format!("{}.json", WORKS_PATH), 200, works_sample);

    let ratings_mock = create_mock(
        &server,
        &format!("{}{}.json", WORKS_PATH, RATINGS_PATH),
        200,
        ratings_sample,
    );

    let client = create_client(&server);
    let book = client
        .book_by_isbn(isbn)
        .await
        .expect("could not get book by isbn");

    isbn_mock.assert();
    works_mock.assert();
    ratings_mock.assert();
    book
}

#[tokio::test]
async fn fetch_book_by_isbn() {
    let isbn = "9780316387316";

    let book = assert_successful_fetch(
        &isbn,
        &get_sample("openlibrary_isbn.json"),
        &get_sample("openlibrary_works.json"),
        &get_sample("openlibrary_ratings.json"),
    )
    .await;
    assert_eq!(create_default_expected_book(), book);
}

#[tokio::test]
async fn fetch_book_by_isbn_with_description_as_string() {
    let isbn = "9780316387316";

    let book = assert_successful_fetch(
        &isbn,
        &get_sample("openlibrary_isbn.json"),
        &get_sample("openlibrary_works_string_description.json"),
        &get_sample("openlibrary_ratings.json"),
    )
    .await;
    assert_eq!(create_default_expected_book(), book);
}

#[tokio::test]
async fn fetch_book_by_isbn_with_no_description() {
    let isbn = "9780316387316";

    let book = assert_successful_fetch(
        &isbn,
        &get_sample("openlibrary_isbn.json"),
        &get_sample("openlibrary_works_no_description.json"),
        &get_sample("openlibrary_ratings.json"),
    )
    .await;
    let mut expected_book = create_default_expected_book();
    expected_book.description = String::new();
    assert_eq!(expected_book, book);
}

#[tokio::test]
async fn fetch_book_by_isbn_with_no_ratings() {
    let isbn = "9780316387316";

    let book = assert_successful_fetch(
        &isbn,
        &get_sample("openlibrary_isbn.json"),
        &get_sample("openlibrary_works.json"),
        &get_sample("openlibrary_no_ratings.json"),
    )
    .await;
    let mut expected_book = create_default_expected_book();
    expected_book.rating = None;
    assert_eq!(expected_book, book);
}

#[tokio::test]
async fn no_book_returned_on_404_from_isbn_call() {
    let isbn = "9780316387316";

    let server = MockServer::start();
    let isbn_mock = create_mock(&server, &format!("{}/{}.json", ISBN_PATH, isbn), 404, "");

    let client = create_client(&server);
    let book = client.book_by_isbn(isbn).await;

    isbn_mock.assert();
    let _returned_error = book
        .err()
        .expect("error not returned when expected for missing book on isbn call");
    assert!(matches!(ClientError::NotFound, _returned_error));
}

#[tokio::test]
async fn no_book_returned_when_works_key_missing() {
    let isbn = "9780316387316";

    let server = MockServer::start();
    let isbn_mock = create_mock(
        &server,
        &format!("{}/{}.json", ISBN_PATH, isbn),
        200,
        &get_sample("openlibrary_isbn_no_works_key.json"),
    );

    let client = create_client(&server);
    let book = client.book_by_isbn(isbn).await;

    isbn_mock.assert();
    let _returned_error = book
        .err()
        .expect("error not returned when expected for missing works key");
    assert!(matches!(ClientError::NotFound, _returned_error));
}

#[tokio::test]
async fn return_rate_limit_error() {
    let isbn = "9780316387316";

    for status_code in [429, 403] {
        let server = MockServer::start();
        let isbn_mock = create_mock(
            &server,
            &format!("{}/{}.json", ISBN_PATH, isbn),
            status_code,
            "",
        );

        let client = create_client(&server);
        let book = client.book_by_isbn(isbn).await;

        isbn_mock.assert();
        let _returned_error = book.err().expect(&format!(
            "error not returned when expected for status {}",
            status_code
        ));
        assert!(matches!(ClientError::RateLimitExceeded, _returned_error));
    }
}

#[tokio::test]
async fn handle_other_http_error() {
    let isbn = "9780316387316";

    for status_code in [400, 500, 503] {
        let server = MockServer::start();
        let isbn_mock = create_mock(
            &server,
            &format!("{}/{}.json", ISBN_PATH, isbn),
            status_code,
            "",
        );

        let client = create_client(&server);
        let book = client.book_by_isbn(isbn).await;

        isbn_mock.assert();
        let returned_error = book.err().expect("error not returned when expected");
        match returned_error {
            ClientError::Http(response_status_code, _) => {
                assert_eq!(status_code, response_status_code);
            }
            _ => {
                panic!("invalid error type returned")
            }
        }
    }
}

#[tokio::test]
#[should_panic]
async fn panics_when_querying_book_by_author_and_title() {
    let author = "Author 1";
    let title = "Title 1";

    let server = MockServer::start();
    let client = create_client(&server);
    let _ = client.book(author, title).await;
}
