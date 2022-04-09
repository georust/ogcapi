use anyhow::Context;
use axum::extract::{Host, OriginalUri};
use axum::http::StatusCode;
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
};
use url::Url;

use crate::Error;

/// Extractor for the the remote url
pub(crate) struct RemoteUrl(pub Url);

#[async_trait]
impl<B> FromRequest<B> for RemoteUrl
where
    B: Send,
{
    type Rejection = Error;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let host = Host::from_request(req)
            .await
            .context("Unabe to extract host")?;

        let uri = OriginalUri::from_request(req)
            .await
            // Infallible
            .unwrap();

        let scheme = if host.0.contains(':') {
            "http"
        } else {
            "https"
        };

        Ok(RemoteUrl(Url::parse(&format!(
            "{}://{}{}",
            scheme, host.0, uri.0
        ))?))
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
