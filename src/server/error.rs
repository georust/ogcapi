use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::common::core::Exception;

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

    /// Custom Exception
    #[error("an ogcapi exception occurred")]
    Exception(StatusCode, String),
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Sqlx(_) | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
            Self::Exception(status, message) => {
                tracing::error!("OGCAPI exception: {}", message);
                (status, message)
            }
        };

        let exception = Exception {
            r#type: format!(
                "https://httpwg.org/specs/rfc7231.html#status.{}",
                status.as_str()
            ),
            status: Some(status.as_u16()),
            detail: Some(message),
            ..Default::default()
        };

        (status, Json(exception)).into_response()
    }
}
