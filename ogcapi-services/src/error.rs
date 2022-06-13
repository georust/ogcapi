use axum::{
    headers::HeaderMap,
    http::{header::CONTENT_TYPE, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use ogcapi_types::common::{media_type::PROBLEM_JSON, Exception};

/// A common error type that can be used throughout the API.
///
/// Can be returned in a `Result` from an API handler function.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Automatically return `500 Internal Server Error` on a `sqlx::Error`.
    // #[error("an error occurred with the database")]
    // Sqlx(#[from] sqlx::Error),

    /// Return `404 Not Found`
    #[error("not found")]
    NotFound,

    /// Return `500 Internal Server Error` on a `anyhow::Error`.
    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),

    /// Return `500 Internal Server Error` on a `url::ParseError`.
    #[error("an internal server error occurred")]
    Url(#[from] url::ParseError),

    /// Return `500 Internal Server Error` on a `serde_qs::Error`.
    #[error("an internal server error occurred")]
    Qs(#[from] serde_qs::Error),

    /// Custom Exception
    #[error("an ogcapi exception occurred")]
    Exception(StatusCode, String),
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Exception(status, _) => *status,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Axum allows you to return `Result` from handler functions, but the error type
/// also must be some sort of response type.
///
/// By default, the generated `Display` impl is used to return a plaintext error message
/// to the client.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            // Self::Sqlx(ref e) => {
            //     tracing::error!("SQLx error: {:?}", e);
            //     (self.status_code(), self.to_string())
            // }
            Self::NotFound => (self.status_code(), self.to_string()),
            Self::Anyhow(ref e) => {
                tracing::error!("Generic error: {:?}", e);
                (self.status_code(), self.to_string())
            }
            Self::Url(ref e) => {
                tracing::error!("Url error: {:?}", e);
                (self.status_code(), self.to_string())
            }
            Self::Qs(ref e) => {
                tracing::error!("Query string error: {:?}", e);
                (self.status_code(), self.to_string())
            }
            Self::Exception(status, message) => {
                tracing::debug!("OGCAPI exception: {}", message);
                (status, message)
            }
        };

        let exception = Exception::new(status.as_u16()).detail(message);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, PROBLEM_JSON.parse().unwrap());

        (status, headers, Json(exception)).into_response()
    }
}
