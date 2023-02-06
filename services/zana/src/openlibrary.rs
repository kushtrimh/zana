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
    description: String,
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

pub struct Client {
    api_url: String,
    http_client: reqwest::Client,
}

impl Client {
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
        let mut book = Book::new(book_response.number_of_pages, &work_response.description);
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
        if status_code == 429 || status_code == 403 {
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

    async fn fetch_book_by_isbn(&self, isbn: &str) -> Result<Option<BookResponse>, ClientError> {
        let response = self
            .send_request(&format!("{}{}/{}.json", self.api_url, ISBN_PATH, isbn))
            .await?;
        if response.status().as_u16() == 404 {
            // todo: add log here later
            return Ok(None);
        }
        Ok(Some(self.handle_response(response).await?.json().await?))
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

    async fn fetch_book(&self, isbn: &str) -> Result<Option<Book>, ClientError> {
        let book_response = match self.fetch_book_by_isbn(isbn).await? {
            Some(book_response) => book_response,
            None => {
                return Ok(None);
            }
        };

        if book_response.works.is_empty() {
            // todo: add debug log here
            return Ok(None);
        }
        let works_path = &book_response.works[0].key;
        let work_response = self.fetch_work(works_path).await?;
        let ratings_response = self.fetch_rating(works_path).await?;

        Ok(Some(self.create_book(
            &book_response,
            &work_response,
            &ratings_response,
        )))
    }
}

#[async_trait]
impl BookClient for Client {
    async fn book_by_isbn(&self, isbn: &str) -> Result<Option<Book>, ClientError> {
        self.fetch_book(isbn).await
    }

    async fn book(&self, _author: &str, _title: &str) -> Result<Option<Book>, ClientError> {
        Ok(None)
    }
}
