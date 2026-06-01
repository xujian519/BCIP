use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebSearchError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error (code {code}): {message}")]
    Api { code: i32, message: String },

    #[error("Rate limited (retry after: {retry_after:?})")]
    RateLimited { retry_after: Option<u64> },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
