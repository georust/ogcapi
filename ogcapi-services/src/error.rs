use axum::headers::HeaderMap;
use axum::http::{header::CONTENT_TYPE, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use ogcapi_types::common::{Exception, MediaType};

/// A common error type that can be used throughout the API.
///
/// Can be returned in a `Result` from an API handler function.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Automatically return `500 Internal Server Error` on a `sqlx::Error`.
    #[error("an error occurred with the database")]
    Sqlx(#[from] sqlx::Error),

    /// Return `500 Internal Server Error` on a `anyhow::Error`.
    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),

    /// Return `500 Internal Server Error` on a `url::ParseError`.
    #[error("an internal server error occurred")]
    Url(#[from] url::ParseError),

    /// Custom Exception
    #[error("an ogcapi exception occurred")]
    Exception(StatusCode, String),
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Sqlx(_) | Self::Anyhow(_) | Self::Url(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Exception(status, _) => *status,
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
            Self::Sqlx(ref e) => {
                tracing::error!("SQLx error: {:?}", e);
                (self.status_code(), self.to_string())
            }
            Self::Anyhow(ref e) => {
                tracing::error!("Generic error: {:?}", e);
                (self.status_code(), self.to_string())
            }
            Self::Url(ref e) => {
                tracing::error!("Generic error: {:?}", e);
                (self.status_code(), self.to_string())
            }
            Self::Exception(status, message) => {
                tracing::error!("OGCAPI exception: {}", message);
                (status, message)
            }
        };

        let exception = Exception::new(status.as_u16()).detail(message);

        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            MediaType::ProblemJSON.to_string().parse().unwrap(),
        );

        (status, headers, Json(exception)).into_response()
    }
}
