use std::fs;

use httpmock::prelude::*;
use httpmock::Mock;

use zana::googlebooks::{Client, Volume};
use zana::ClientError;

const API_KEY: &str = "b85a45ddd5a99124cf4ec9a74f93fcf1";
const VOLUME_PATH: &str = "/books/v1/volumes";

fn get_sample(sample: &str) -> String {
    fs::read_to_string(format!("tests/sample/{}", sample)).expect("could not read sample file")
}

fn create_client(server: &MockServer) -> Client {
    Client::new(API_KEY, &format!("http://{}", &server.address())).expect("could not create client")
}

fn assert_volume_equality(volume: Volume) {
    let volume_item = &volume.items.expect("item should exist")[0];
    let volume_info = &volume_item.info;

    assert_eq!("Joe Abercrombie", volume_info.authors[0]);
    assert_eq!("The Blade Itself", volume_info.title);
    assert_eq!(560, volume_info.page_count);
    assert_eq!(3.5, volume_info.average_rating);
    assert_eq!(107, volume_info.ratings_count);
    assert_eq!(
        "The first novel in the First Law Trilogy",
        volume_info.description
    );
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
async fn retrieve_volume_by_isbn() {
    let isbn = "9780316387316";

    let server = MockServer::start();
    let m = create_mock(
        &server,
        &format!("isbn:{}", isbn),
        200,
        &get_sample("googlebooks_volume.json"),
    );

    let client = create_client(&server);
    let volume = client
        .volume_by_isbn(isbn)
        .await
        .expect("could not get volume by isbn");

    m.assert();
    assert_volume_equality(volume);
}

#[tokio::test]
async fn retrieve_volume_by_name_and_author() {
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
    let volume = client
        .volume(author, title)
        .await
        .expect("could not get volume by title and author");

    m.assert();
    assert_volume_equality(volume);
}

#[tokio::test]
async fn handle_empty_volume_response() {
    let isbn = "9780316387316";

    let server = MockServer::start();
    let m = create_mock(&server, &format!("isbn:{}", isbn), 200, "{}");

    let client = create_client(&server);
    let volume = client
        .volume_by_isbn(isbn)
        .await
        .expect("could not get volume by isbn");

    m.assert();

    assert!(volume.items.is_none())
}

#[tokio::test]
async fn return_rate_limit_error() {
    let isbn = "9780316387316";

    for status_code in [429, 403] {
        let server = MockServer::start();
        let m = create_mock(&server, &format!("isbn:{}", isbn), status_code, "");

        let client = create_client(&server);
        let volume = client.volume_by_isbn(isbn).await;

        m.assert();
        let _returned_error = volume.err().expect(&format!(
            "error not returned when expected for status {}",
            status_code
        ));
        assert!(matches!(ClientError::RateLimitExceeded, _returned_error));
    }
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
    let volume = client.volume_by_isbn(isbn).await;

    m.assert();
    let returned_error = volume
        .err()
        .expect("error not returned when expected");
    match returned_error {
        ClientError::Http(status_code, _) => {
            assert_eq!(expected_status_code, status_code);
        }
        _ => {
            panic!("invalid error type returned")
        }
    }
}
