/*!
Queries book data from OpenLibrary using the [`Client`](struct@Client)
implementation of [`BookClient`](trait@BookClient).

Three different API calls are made to query all the needed data, and their data
is then aggregated to return a single book.
1. At first it queries the book by ISBN.
2. Then queries the `work` endpoint, to retrieve more data about the book,
its authors and description.
3. Queries ratings.

See example [here](../index.html#example-1).
 */

use async_trait::async_trait;
use serde::Deserialize;

use crate::{create_http_client, Book, BookClient, ClientError, Rating};

const ISBN_PATH: &str = "/isbn";
const RATINGS_PATH: &str = "/ratings";

#[derive(Deserialize, Debug)]
struct BookResponse {
    number_of_pages: u32,
    works: Vec<WorkIdentifier>,
}

#[derive(Deserialize, Debug)]
struct WorkIdentifier {
    key: String,
}

#[derive(Deserialize, Debug)]
struct WorkResponse {
    description: Option<Description>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Description {
    String(String),
    Map(DescriptionMap),
}

#[derive(Deserialize, Debug)]
struct DescriptionMap {
    value: String,
}

#[derive(Deserialize, Debug)]
struct RatingResponse {
    summary: RatingSummary,
}

#[derive(Deserialize, Debug)]
struct RatingSummary {
    average: Option<f32>,
    count: u32,
}

/// Client used to retrieve data from OpenLibrary API.
pub struct Client {
    api_url: String,
    http_client: reqwest::Client,
}

impl Client {
    /// Returns a new client that will make requests to the given API URL.
    ///
    /// No API key is required for OpenLibrary.
    pub fn new(api_url: &str) -> Result<Self, ClientError> {
        let http_client = create_http_client()?;
        Ok(Client {
            api_url: String::from(api_url),
            http_client,
        })
    }

    fn create_book(
        &self,
        book_response: &BookResponse,
        work_response: &WorkResponse,
        rating_response: &RatingResponse,
    ) -> Book {
        let description = match &work_response.description {
            Some(description) => match description {
                Description::String(description_string) => description_string,
                Description::Map(description_map) => &description_map.value,
            },
            None => "",
        };
        let mut book = Book::new(book_response.number_of_pages, description);
        if let Some(average_rating) = rating_response.summary.average {
            book.rating = Some(Rating::new(average_rating, rating_response.summary.count));
        }
        book
    }

    async fn handle_response(
        &self,
        response: reqwest::Response,
    ) -> Result<reqwest::Response, ClientError> {
        let status_code = response.status().as_u16();
        if status_code == 404 {
            Err(ClientError::NotFound)
        } else if status_code == 429 || status_code == 403 {
            Err(ClientError::RateLimitExceeded)
        } else if status_code < 200 || status_code >= 300 {
            let response_body = response.text().await?;
            Err(ClientError::Http(status_code, response_body))
        } else {
            Ok(response)
        }
    }

    async fn send_request(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client.get(url).send().await
    }

    async fn fetch_book_by_isbn(&self, isbn: &str) -> Result<BookResponse, ClientError> {
        let response = self
            .send_request(&format!("{}{}/{}.json", self.api_url, ISBN_PATH, isbn))
            .await?;
        if response.status().as_u16() == 404 {
            log::debug!("book with ISBN({}) not found on Open Library", isbn);
            return Err(ClientError::NotFound);
        }
        Ok(self.handle_response(response).await?.json().await?)
    }

    async fn fetch_work(&self, work_path: &str) -> Result<WorkResponse, ClientError> {
        let response = self
            .send_request(&format!("{}{}.json", self.api_url, work_path))
            .await?;
        Ok(self.handle_response(response).await?.json().await?)
    }

    async fn fetch_rating(&self, work_path: &str) -> Result<RatingResponse, ClientError> {
        let response = self
            .send_request(&format!(
                "{}{}{}.json",
                self.api_url, work_path, RATINGS_PATH
            ))
            .await?;
        Ok(self.handle_response(response).await?.json().await?)
    }

    async fn fetch_book(&self, isbn: &str) -> Result<Book, ClientError> {
        let book_response = self.fetch_book_by_isbn(isbn).await?;

        if book_response.works.is_empty() {
            log::debug!(
                "no works identifier found for book with ISBN({}) on Open Library",
                isbn
            );
            return Err(ClientError::NotFound);
        }
        let works_path = &book_response.works[0].key;
        let work_response = self.fetch_work(works_path).await?;
        let ratings_response = self.fetch_rating(works_path).await?;

        Ok(self.create_book(&book_response, &work_response, &ratings_response))
    }
}

#[async_trait]
impl BookClient for Client {
    /// Returns a book by ISBN.
    ///
    /// Queries 3 different endpoints to retrieve all the needed data.
    /// 1. /books
    /// 2. /works
    /// 3. /ratings
    ///
    /// If an error occurs with the communication, an HTTP status code that is not 200 is returned,
    /// the book is not found, or the rate limit is exceeded then an error is returned.
    async fn book_by_isbn(&self, isbn: &str) -> Result<Book, ClientError> {
        self.fetch_book(isbn).await
    }

    /// The method to retrieve book information by author and title is not supported by OpenLibrary,
    /// so this method calls `unimplemented!` directly.
    async fn book(&self, _author: &str, _title: &str) -> Result<Book, ClientError> {
        unimplemented!("not supported by third-party");
    }
}
