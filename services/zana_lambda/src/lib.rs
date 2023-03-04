/*!
_zana_lambda_ provides functionality to fetch books data and is
meant to be used only as a Rust AWS HTTP Lambda.

Functionality to fetch books is all provided by [`zana`](zana).
This crate provides functionality to fetch Book client parameters via AWS Parameter Store,
deals with error handling, and with request parameters required to fetch books from different clients.

[`Client`](struct@book::Client) is used to query book data from all providers that [`zana`](zana) supports.

## Example

```
use zana::{googlebooks, openlibrary};
use zana_lambda::http::{RequestType, ResponseError, failure_response};
use zana_lambda::book::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let googlebooks_client = match googlebooks::Client::new("google-books-api-key", "google-books-api-url") {
        Ok(client) => Box::new(client),
        Err(err) => panic!("could not create client"),
    };

    let openlibrary_client = match openlibrary::Client::new("open-library-api-url") {
        Ok(client) => Box::new(client),
        Err(err) => panic!("could not create client"),
    };

    let isbn = "9781591026419";

    let client = Client::new(googlebooks_client, openlibrary_client);
    match client.fetch_book(&RequestType::OpenLibrary, isbn, "", "").await {
        Ok(book) => println!("book found ({}: {:?})", isbn, &book),
        Err(err) => eprintln!("could not fetch book by ISBN {:?}", err),
    };

    Ok(())
}
```
*/
pub mod book;
pub mod http;
pub mod params;
