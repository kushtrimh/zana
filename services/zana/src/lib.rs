pub mod googlebooks;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("error coming from internal http client")]
    InternalClient(#[from] reqwest::Error),
    #[error("rate limit exceeded for external service")]
    RateLimitExceeded,
    #[error("generic http error that contains status code and response body")]
    Http(u16, String),
}
