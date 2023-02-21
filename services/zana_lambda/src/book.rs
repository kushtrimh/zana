use crate::http::{RequestType, ResponseError};
use zana::{Book, BookClient};

pub type BookApiClient = dyn BookClient + Send + Sync;

pub struct Client {
    googlebooks_client: Box<BookApiClient>,
    openlibrary_client: Box<BookApiClient>,
}

impl Client {
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

    pub async fn fetch_book(
        &self,
        request_type: &RequestType,
        isbn: &str,
        title: &str,
        author: &str,
    ) -> Result<Book, ResponseError> {
        if !isbn.is_empty() {
            log::debug!("fetching book by isbn {} for {:?}", isbn, &request_type);
            self.fetch_by_isbn(&request_type, &isbn).await
        } else if !author.is_empty() && !title.is_empty() {
            log::debug!(
                "fetching book by title {} and author {} for {:?}",
                title,
                author,
                &request_type
            );
            self.fetch_by_title_and_author(&request_type, &title, &author)
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

    #[test]
    fn create_success_response() {}

    #[test]
    fn create_failure_response() {}

    #[test]
    fn create_response_from_client_error() {}
}
