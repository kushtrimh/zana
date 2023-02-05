use serde::Deserialize;

use crate::ClientError;

const VOLUMES_PATH: &str = "/books/v1/volumes";

#[derive(Deserialize, Debug)]
pub struct Volume {
    pub items: Option<Vec<VolumeItem>>,
}

#[derive(Deserialize, Debug)]
pub struct VolumeItem {
    #[serde(rename(deserialize = "volumeInfo"))]
    pub info: VolumeInfo,
}

#[derive(Deserialize, Debug)]
pub struct VolumeInfo {
    pub title: String,
    pub authors: Vec<String>,
    pub description: String,
    #[serde(rename(deserialize = "pageCount"))]
    pub page_count: u32,
    #[serde(rename(deserialize = "averageRating"))]
    pub average_rating: f32,
    #[serde(rename(deserialize = "ratingsCount"))]
    pub ratings_count: u32,
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

    pub async fn volume_by_isbn(&self, isbn: &str) -> Result<Volume, ClientError> {
        self.fetch_volume(&format!("isbn:{}", isbn)).await
    }

    pub async fn volume(&self, author: &str, title: &str) -> Result<Volume, ClientError> {
        self.fetch_volume(&format!("inauthor:{} intitle:{}", author, title))
            .await
    }

    async fn fetch_volume(&self, query: &str) -> Result<Volume, ClientError> {
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

        let mut volume: Volume = response.json().await?;

        if let Some(items) = &volume.items {
            if items.is_empty() {
                volume.items = None
            }
        }
        Ok(volume)
    }
}
