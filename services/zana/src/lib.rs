use thiserror::Error;

pub mod googlebooks;
pub mod openlibrary;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("error coming from internal http client")]
    InternalClient(#[from] reqwest::Error),
    #[error("rate limit exceeded for external service")]
    RateLimitExceeded,
    #[error("generic http error that contains status code and response body")]
    Http(u16, String),
}

#[derive(Debug)]
pub struct Book {
    pub page_count: u32,
    pub description: String,
    pub rating: Option<Rating>,
}

#[derive(Debug)]
pub struct Rating {
    pub average_rating: f32,
    pub ratings_count: u32,
}

impl Book {
    pub fn new(page_count: u32, description: &str) -> Book {
        Book {
            page_count,
            description: String::from(description),
            rating: None,
        }
    }

    pub fn new_with_rating(page_count: u32, description: &str, rating: Rating) -> Book {
        let mut book = Book::new(page_count, description);
        book.rating = Some(rating);
        book
    }
}

impl Rating {
    pub fn new(average_rating: f32, ratings_count: u32) -> Rating {
        Rating {
            average_rating,
            ratings_count,
        }
    }
}
