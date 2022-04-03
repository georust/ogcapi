use axum::http::StatusCode;
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
};
use url::Url;

use crate::Error;

pub(crate) struct RemoteUrl(pub Url);

#[async_trait]
impl<B> FromRequest<B> for RemoteUrl
where
    B: Send,
{
    type Rejection = Error;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let headers = req.headers();
        if let Some(url) = headers
            .get("Forwarded")
            .and_then(|header| {
                header.to_str().ok().and_then(|value| {
                    value.split(';').find_map(|key_equals_value| {
                        let parts = key_equals_value.split('=').collect::<Vec<_>>();
                        if parts.len() == 2 && parts[0].eq_ignore_ascii_case("for") {
                            Some(parts[1])
                        } else {
                            headers.get("X-Forwarded-For").and_then(|header| {
                                header
                                    .to_str()
                                    .ok()
                                    .and_then(|value| value.split(',').next())
                            })
                        }
                    })
                })
            })
            .and_then(|value| Url::parse(value).ok())
        {
            Ok(RemoteUrl(url))
        } else {
            Err(Error::Exception(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to extract remote URL".to_string(),
            ))
        }
    }
}

/// Extractor that deserializes query strings into some type `T` with [`serde_qs`]
pub(crate) struct Qs<T>(pub(crate) T);

#[axum::async_trait]
impl<B, T> FromRequest<B> for Qs<T>
where
    B: Send,
    T: serde::de::DeserializeOwned,
{
    type Rejection = Error;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let qs = req.uri().query().unwrap_or("");
        match serde_qs::from_str(qs) {
            Ok(query) => Ok(Self(query)),
            Err(e) => Err(Error::Exception(StatusCode::BAD_REQUEST, e.to_string())),
        }
    }
}
