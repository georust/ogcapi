use thiserror::Error;

/// Errors which can occur when fetching, and decoding STAC entities.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Encountered a request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Encountered a conformance error: `{0}`")]
    UnknownConformance(String),

    #[error("Encountered url parse error")]
    UrlError(#[from] url::ParseError),

    #[error("Encountered a query string error")]
    QueryStringError(#[from] serde_qs::Error),

    #[error("Encountered a serialization error: {0}")]
    DeserializationError(serde_json::Error),

    #[error("Encountered a client error: {0}")]
    ClientError(String),
}
