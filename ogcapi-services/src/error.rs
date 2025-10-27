use axum::{
    Json,
    http::{StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use hyper::HeaderMap;

use ogcapi_types::common::{Exception, media_type::PROBLEM_JSON};
use tracing::error;

/// A common error type that can be used throughout the API.
///
/// Can be returned in a `Result` from an API handler function.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    // /// Automatically return `500 Internal Server Error` on a `sqlx::Error`.
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

    #[error("an OGC API exception occurred")]
    OgcApiException(#[from] Exception),
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Exception(status, _) => *status,
            Self::OgcApiException(exception) => exception
                .status
                .and_then(|status| StatusCode::from_u16(status).ok())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
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
        let status = self.status_code();
        let exception = Exception::from(self);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, PROBLEM_JSON.parse().unwrap());

        (status, headers, Json(exception)).into_response()
    }
}

impl From<Error> for Exception {
    fn from(value: Error) -> Self {
        let (status, message) = match value {
            // Self::Sqlx(ref e) => {
            //     tracing::error!("SQLx error: {:?}", e);
            //     (self.status_code(), self.to_string())
            // }
            Error::NotFound => (value.status_code(), value.to_string()),
            Error::Anyhow(ref e) => {
                tracing::error!("Generic error: {:?}", e);
                (value.status_code(), e.to_string())
            }
            Error::Url(ref e) => {
                tracing::error!("Url error: {:?}", e);
                (value.status_code(), e.to_string())
            }
            Error::Qs(ref e) => {
                tracing::error!("Query string error: {:?}", e);
                (value.status_code(), e.to_string())
            }
            Error::Exception(status, message) => {
                tracing::debug!("OGCAPI exception: {}", message);
                (status, message)
            }
            Error::OgcApiException(exception) => {
                return exception;
            }
        };

        Exception::new(status.as_u16()).detail(message)
    }
}

/// Helper function to read-lock a RwLock, recovering from poisoning if necessary.
pub fn read_lock<T>(mutex: &std::sync::RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    match mutex.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("Mutex was poisoned, attempting to recover.");
            poisoned.into_inner()
        }
    }
}

/// Helper function to read-lock a RwLock, recovering from poisoning if necessary.
pub fn write_lock<T>(mutex: &std::sync::RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    match mutex.write() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("Mutex was poisoned, attempting to recover.");
            poisoned.into_inner()
        }
    }
}
