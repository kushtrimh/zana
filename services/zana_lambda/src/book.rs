use crate::http::{RequestType, ResponseError};
use zana::{googlebooks, openlibrary, Book, BookClient, ClientError};

pub type BookApiClient = dyn BookClient + Send + Sync;

pub struct Client {
    googlebooks_client: Box<BookApiClient>,
    openlibrary_client: Box<BookApiClient>,
}

impl Client {
    pub fn new(
        google_books_data: (&str, &str),
        open_library_api_url: &str,
    ) -> Result<Self, ClientError> {
        let (googlebooks_api_key, googlebooks_api_url) = google_books_data;
        let googlebooks_client =
            googlebooks::Client::new(googlebooks_api_key, googlebooks_api_url)?;
        let googlebooks_client = Box::new(googlebooks_client);

        let openlibrary_client = openlibrary::Client::new(open_library_api_url)?;
        let openlibrary_client = Box::new(openlibrary_client);

        Ok(Self {
            openlibrary_client,
            googlebooks_client,
        })
    }

    fn client_from_type(&self, request_type: &RequestType) -> &BookApiClient {
        match request_type {
            RequestType::GoogleBooks => self.googlebooks_client.as_ref(),
            RequestType::OpenLibrary => self.openlibrary_client.as_ref(),
        }
    }

    pub async fn fetch_by_isbn(
        &self,
        request_type: &RequestType,
        isbn: &str,
    ) -> Result<Book, ResponseError> {
        log::debug!("sending volume query request for isbn: {}", isbn);
        Ok(self
            .client_from_type(request_type)
            .book_by_isbn(isbn)
            .await?)
    }

    pub async fn fetch_by_title_and_author(
        &self,
        request_type: &RequestType,
        title: &str,
        author: &str,
    ) -> Result<Book, ResponseError> {
        log::debug!(
            "sending volume query request for title: {} and author: {}",
            title,
            author
        );
        Ok(self
            .client_from_type(request_type)
            .book(author, title)
            .await?)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn create_success_response() {}

    #[test]
    fn create_failure_response() {}

    #[test]
    fn create_response_from_client_error() {}
}
