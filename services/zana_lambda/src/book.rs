/*!
Queries book data from providers supported by [`zana`](zana).
It uses [`RequestType`](enum@RequestType) to decide on which provider to use.
*/
use crate::http::{RequestType, ResponseError};
use zana::{Book, BookClient};

pub type BookApiClient = dyn BookClient + Send + Sync;

/// Client used to fetch books from different providers.
/// Acts as a container for different clients supported by [`zana`](zana).
pub struct Client {
    googlebooks_client: Box<BookApiClient>,
    openlibrary_client: Box<BookApiClient>,
}

impl Client {
    /// Returns a new client that contains [`zana`](zana) clients for each provider.
    pub fn new(
        googlebooks_client: Box<BookApiClient>,
        openlibrary_client: Box<BookApiClient>,
    ) -> Self {
        Self {
            googlebooks_client,
            openlibrary_client,
        }
    }

    fn client_from_type(&self, request_type: &RequestType) -> &BookApiClient {
        match request_type {
            RequestType::GoogleBooks => self.googlebooks_client.as_ref(),
            RequestType::OpenLibrary => self.openlibrary_client.as_ref(),
        }
    }

    /// Returns a book by ISBN
    ///
    /// Based on [`RequestType`](enum@RequestType) the correct provider will be used
    /// to fetch the book by ISBN.
    /// If any there are communication problems, an HTTP status code that is not 200 is returned,
    /// or the book is not found, an error is returned.
    pub async fn fetch_by_isbn(
        &self,
        request_type: &RequestType,
        isbn: &str,
    ) -> Result<Book, ResponseError> {
        tracing::debug!("sending volume query request for isbn: {}", isbn);
        Ok(self
            .client_from_type(request_type)
            .book_by_isbn(isbn)
            .await?)
    }

    /// Returns a book by title and author
    ///
    /// Based on [`RequestType`](enum@RequestType) the correct provider will be used
    /// to fetch the book by title and author.
    /// If any there are communication problems, an HTTP status code that is not 200 is returned,
    /// or the book is not found, an error is returned.
    pub async fn fetch_by_title_and_author(
        &self,
        request_type: &RequestType,
        title: &str,
        author: &str,
    ) -> Result<Book, ResponseError> {
        tracing::debug!(
            "sending volume query request for title: {} and author: {}",
            title,
            author
        );
        Ok(self
            .client_from_type(request_type)
            .book(author, title)
            .await?)
    }

    /// Returns a book
    ///
    /// Based on [`RequestType`](enum@RequestType) the correct provider will be used
    /// to fetch the book by either ISBN, or title and author.
    /// ISBN has precedence over title and author.
    /// If ISBN, title and author are all empty, and error is returned.
    /// If any there are communication problems, an HTTP status code that is not 200 is returned,
    /// or the book is not found, an error is returned.
    pub async fn fetch_book(
        &self,
        request_type: &RequestType,
        isbn: &str,
        title: &str,
        author: &str,
    ) -> Result<Book, ResponseError> {
        if !isbn.is_empty() {
            tracing::debug!("fetching book by isbn {} for {:?}", isbn, &request_type);
            self.fetch_by_isbn(request_type, isbn).await
        } else if !author.is_empty() && !title.is_empty() {
            tracing::debug!(
                "fetching book by title {} and author {} for {:?}",
                title,
                author,
                &request_type
            );
            self.fetch_by_title_and_author(request_type, title, author)
                .await
        } else {
            Err(ResponseError::MissingParameter(String::from(
                "Either ISBN or title and author must be provided",
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::book::Client;
    use crate::http::{RequestType, ResponseError};
    use async_trait::async_trait;
    use zana::{Book, BookClient, ClientError};

    struct TestBookClient {
        isbn: String,
        title: String,
        author: String,
        pages: u32,
        description: String,
    }

    impl TestBookClient {
        fn default() -> Self {
            Self::new("", "", "", 0, "")
        }

        fn new(isbn: &str, title: &str, author: &str, pages: u32, description: &str) -> Self {
            Self {
                isbn: String::from(isbn),
                pages,
                title: String::from(title),
                author: String::from(author),
                description: String::from(description),
            }
        }

        fn new_with_isbn(isbn: &str, pages: u32, description: &str) -> Self {
            Self::new(isbn, "", "", pages, description)
        }

        fn new_with_title_and_author(
            title: &str,
            author: &str,
            pages: u32,
            description: &str,
        ) -> Self {
            Self::new("", title, author, pages, description)
        }
    }

    #[async_trait]
    impl BookClient for TestBookClient {
        async fn book_by_isbn(&self, isbn: &str) -> Result<Book, ClientError> {
            if self.isbn == isbn {
                Ok(Book::new(self.pages, &self.description))
            } else {
                Err(ClientError::NotFound)
            }
        }

        async fn book(&self, author: &str, title: &str) -> Result<Book, ClientError> {
            if self.title == title && self.author == author {
                Ok(Book::new(self.pages, &self.description))
            } else {
                Err(ClientError::NotFound)
            }
        }
    }

    #[tokio::test]
    async fn return_error_when_all_parameters_are_empty() {
        let gb_client = TestBookClient::default();
        let op_client = TestBookClient::default();
        let client = Client::new(Box::new(gb_client), Box::new(op_client));
        match client
            .fetch_book(&RequestType::OpenLibrary, "", "", "")
            .await
        {
            Ok(_) => panic!("error expected when all parameters are empty"),
            Err(err) => match err {
                ResponseError::MissingParameter(message) => {
                    assert_eq!("Either ISBN or title and author must be provided", message)
                }
                _ => panic!("invalid error type returned"),
            },
        }
    }

    #[tokio::test]
    async fn fetch_book_by_isbn() {
        let isbn = "9781591026419";
        let pages = 100;
        let description = "Book description";

        let gb_client = TestBookClient::default();
        let op_client = TestBookClient::new_with_isbn(isbn, pages, description);
        let client = Client::new(Box::new(gb_client), Box::new(op_client));
        let returned_book = client
            .fetch_book(&RequestType::OpenLibrary, isbn, "", "")
            .await
            .expect("could not retrieve book");

        assert_eq!(Book::new(pages, description), returned_book);
    }

    #[tokio::test]
    async fn fetch_book_by_title_and_author() {
        let title = "Book title";
        let author = "Author Rothua";
        let pages = 100;
        let description = "Book description";

        let gb_client = TestBookClient::default();
        let op_client =
            TestBookClient::new_with_title_and_author(title, author, pages, description);
        let client = Client::new(Box::new(gb_client), Box::new(op_client));
        let returned_book = client
            .fetch_book(&RequestType::OpenLibrary, "", title, author)
            .await
            .expect("could not retrieve book");

        assert_eq!(Book::new(pages, description), returned_book);
    }
}
