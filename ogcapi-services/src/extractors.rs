use anyhow::{Context, Result};
use axum::{
    extract::{FromRequestParts, OriginalUri},
    http::{StatusCode, request::Parts},
};
use url::Url;

use crate::Error;

/// Extractor for the remote URL
pub(crate) struct RemoteUrl(pub Url);

impl<S> FromRequestParts<S> for RemoteUrl
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let uri = OriginalUri::from_request_parts(parts, state).await.unwrap();

        let url = if let Ok(url) = std::env::var("PUBLIC_URL") {
            format!(
                "{}{}",
                url.trim_end_matches('/'),
                uri.path_and_query().unwrap()
            )
        } else if uri.0.scheme().is_some() {
            uri.0.to_string()
        } else {
            let host = host_from_request_parts(parts)
                .await?
                .context("Unable to extract host")?;

            let proto = parts
                .headers
                .get("X-Forwarded-Proto")
                .and_then(|f| f.to_str().ok())
                .unwrap_or("http");

            format!("{}://{}{}", proto, host, uri.0)
        };

        Ok(RemoteUrl(Url::parse(&url)?))
    }
}

/// From `axum_extra::extract::Host`: Cf. <https://github.com/tokio-rs/axum/issues/3442>
async fn host_from_request_parts(parts: &mut Parts) -> Result<Option<String>> {
    use axum::http;
    use hyper::{HeaderMap, header::FORWARDED};

    const X_FORWARDED_HOST_HEADER_KEY: &str = "X-Forwarded-Host";

    #[allow(warnings)]
    fn parse_forwarded(headers: &HeaderMap) -> Option<&str> {
        // if there are multiple `Forwarded` `HeaderMap::get` will return the first one
        let forwarded_values = headers.get(FORWARDED)?.to_str().ok()?;

        // get the first set of values
        let first_value = forwarded_values.split(',').nth(0)?;

        // find the value of the `host` field
        first_value.split(';').find_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            key.trim()
                .eq_ignore_ascii_case("host")
                .then(|| value.trim().trim_matches('"'))
        })
    }

    fn parse_authority(auth: &http::uri::Authority) -> &str {
        auth.as_str()
            .rsplit('@')
            .next()
            .expect("split always has at least 1 item")
    }

    if let Some(host) = parse_forwarded(&parts.headers) {
        return Ok(Some(host.to_owned()));
    }

    if let Some(host) = parts
        .headers
        .get(X_FORWARDED_HOST_HEADER_KEY)
        .and_then(|host| host.to_str().ok())
    {
        return Ok(Some(host.to_owned()));
    }

    if let Some(host) = parts
        .headers
        .get(http::header::HOST)
        .and_then(|host| host.to_str().ok())
    {
        return Ok(Some(host.to_owned()));
    }

    if let Some(authority) = parts.uri.authority() {
        return Ok(Some(parse_authority(authority).to_owned()));
    }

    Ok(None)
}

/// Extractor that deserializes query strings into some type `T` with [`serde_qs`]
pub(crate) struct Qs<T>(pub(crate) T);

impl<S, T> FromRequestParts<S> for Qs<T>
where
    S: Send + Sync,
    T: serde::de::DeserializeOwned,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let qs = parts.uri.query().unwrap_or("");
        match serde_qs::from_str(qs) {
            Ok(query) => Ok(Self(query)),
            Err(e) => Err(Error::ApiException(
                (StatusCode::BAD_REQUEST, e.to_string()).into(),
            )),
        }
    }
}
