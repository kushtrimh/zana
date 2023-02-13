mod util;

use httpmock::prelude::*;
use httpmock::Mock;
use zana::{Book, BookClient};

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

fn assert_book_equality(book: Book) {
    let rating = book.rating.expect("ratings should exist");

    assert_eq!(542, book.page_count);
    assert_eq!(
        book.description,
        "Logen Ninefingers, infamous barbarian, has finally run out of luck.",
    );
    assert_eq!(4.5, rating.average_rating);
    assert_eq!(23, rating.ratings_count);
}

#[tokio::test]
async fn fetch_book_by_isbn() {
    let isbn = "9780316387316";

    let server = MockServer::start();
    let isbn_mock = create_mock(
        &server,
        &format!("{}/{}.json", ISBN_PATH, isbn),
        200,
        &get_sample("openlibrary_isbn.json"),
    );

    let works_mock = create_mock(
        &server,
        &format!("{}.json", WORKS_PATH),
        200,
        &get_sample("openlibrary_works.json"),
    );

    let ratings_mock = create_mock(
        &server,
        &format!("{}{}.json", WORKS_PATH, RATINGS_PATH),
        200,
        &get_sample("openlibrary_ratings.json"),
    );

    let client = create_client(&server);
    let book = client
        .book_by_isbn(isbn)
        .await
        .expect("could not get book by isbn")
        .expect("book should not be empty");

    isbn_mock.assert();
    works_mock.assert();
    ratings_mock.assert();

    assert_book_equality(book);
}

// Todo: add tests where description is just a string
