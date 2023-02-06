use async_trait::async_trait;
use serde::Deserialize;

use crate::{Book, BookClient, ClientError, Rating};

const VOLUMES_PATH: &str = "/books/v1/volumes";

#[derive(Deserialize, Debug)]
struct Volume {
    items: Option<Vec<VolumeItem>>,
}

#[derive(Deserialize, Debug)]
struct VolumeItem {
    #[serde(rename(deserialize = "volumeInfo"))]
    info: VolumeInfo,
}

#[derive(Deserialize, Debug)]
struct VolumeInfo {
    #[serde(rename(deserialize = "title"))]
    _title: String,
    #[serde(rename(deserialize = "authors"))]
    _authors: Vec<String>,
    description: String,
    #[serde(rename(deserialize = "pageCount"))]
    page_count: u32,
    #[serde(rename(deserialize = "averageRating"))]
    average_rating: f32,
    #[serde(rename(deserialize = "ratingsCount"))]
    ratings_count: u32,
}

fn create_book(items: Vec<VolumeItem>) -> Option<Book> {
    if items.is_empty() {
        return None;
    }

    let volume_item = &items[0];
    let volume_info = &volume_item.info;

    let rating = Rating::new(volume_info.average_rating, volume_info.ratings_count);
    Some(Book::new_with_rating(
        volume_info.page_count,
        &volume_info.description,
        rating,
    ))
}

pub struct Client {
    api_key: String,
    api_url: String,
    http_client: reqwest::Client,
}

impl Client {
    pub fn new(api_key: &str, api_url: &str) -> Result<Self, ClientError> {
        let version: &str = option_env!("CARGO_PKG_VERSION").unwrap_or("1.0.0");

        let http_client = reqwest::Client::builder()
            .user_agent(format!("zana/{} (gzip)", version))
            .build()?;
        Ok(Client {
            api_key: String::from(api_key),
            api_url: String::from(api_url),
            http_client,
        })
    }

    async fn fetch_book(&self, query: &str) -> Result<Option<Book>, ClientError> {
        let query_list: Vec<(&str, &str)> = vec![
            ("key", &self.api_key),
            ("maxResults", "1"),
            ("fields", "items"),
            ("q", query),
        ];

        let response = self
            .http_client
            .get(format!("{}{}", self.api_url, VOLUMES_PATH))
            .header("Accept-Encoding", "gzip")
            .query(&query_list)
            .send()
            .await?;

        let status_code = response.status().as_u16();
        if status_code == 429 || status_code == 403 {
            return Err(ClientError::RateLimitExceeded);
        } else if status_code < 200 || status_code >= 300 {
            let response_body = response.text().await?;
            return Err(ClientError::Http(status_code, response_body));
        }

        let volume: Volume = response.json().await?;

        if let Some(items) = volume.items {
            return Ok(create_book(items));
        }
        Ok(None)
    }
}

#[async_trait]
impl BookClient for Client {
    async fn book_by_isbn(&self, isbn: &str) -> Result<Option<Book>, ClientError> {
        self.fetch_book(&format!("isbn:{}", isbn)).await
    }

    async fn book(&self, author: &str, title: &str) -> Result<Option<Book>, ClientError> {
        self.fetch_book(&format!("inauthor:{} intitle:{}", author, title))
            .await
    }
}
