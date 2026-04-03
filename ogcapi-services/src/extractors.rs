use anyhow::Context;
use axum::{
    extract::{FromRequestParts, OriginalUri},
    http::{StatusCode, request::Parts},
};
use axum_extra::TypedHeader;
use headers::Host;
use url::Url;

use crate::Error;

/// Extractor for the remote URL.
/// This should be the <scheme>://<BASE_URL><ROUTER_PATH>?<QUERY> of the original request, even if the API is behind a reverse proxy.
/// The `PUBLIC_URL` environment variable can be set to override the base URL (useful if the API is behind a reverse proxy that doesn't forward the original host or scheme).
pub(crate) struct RemoteUrl(pub Url);

static PUBLIC_URL: &str = "PUBLIC_URL";

impl<S> FromRequestParts<S> for RemoteUrl
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let uri = OriginalUri::from_request_parts(parts, state).await.unwrap();

        let url = if let Ok(url) = std::env::var(PUBLIC_URL) {
            format!(
                "{}{}",
                url.trim_end_matches('/'),
                uri.path_and_query().unwrap()
            )
        } else if uri.0.scheme().is_some() {
            uri.0.to_string()
        } else {
            let host = TypedHeader::<Host>::from_request_parts(parts, state)
                .await
                .context("Unable to extract host")?;

            let proto = parts
                .headers
                .get("X-Forwarded-Proto")
                .and_then(|f| f.to_str().ok())
                .unwrap_or("http");

            if let Some(port) = host.port() {
                format!("{}://{}:{}{}", proto, host.hostname(), port, uri.0)
            } else {
                format!("{}://{}{}", proto, host.hostname(), uri.0)
            }
        };

        Ok(RemoteUrl(Url::parse(&url)?))
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, request::Builder};
    use std::env;

    #[tokio::test]
    async fn it_extracts_remote_urls_all_branches() {
        async fn request_to_remote_url_str(request_builder: Builder) -> String {
            let (mut parts, _) = request_builder.body(()).unwrap().into_parts();
            RemoteUrl::from_request_parts(&mut parts, &())
                .await
                .unwrap()
                .0
                .as_str()
                .to_string()
        }

        // PUBLIC_URL set
        unsafe {
            env::set_var(PUBLIC_URL, "https://public.example.com/");
        }
        assert_eq!(
            request_to_remote_url_str(Request::builder().uri("/some/path?x=1")).await,
            "https://public.example.com/some/path?x=1"
        );

        unsafe {
            env::set_var(PUBLIC_URL, "https://public.example.com/subdir/");
        }
        assert_eq!(
            request_to_remote_url_str(Request::builder().uri("/some/path?x=1")).await,
            "https://public.example.com/subdir/some/path?x=1"
        );

        unsafe {
            env::remove_var(PUBLIC_URL);
        }

        // URI already contains scheme
        let req = Request::builder()
            .uri("https://api.example.com/thing?x=2")
            .body(())
            .unwrap();
        let (mut parts, _) = req.into_parts();
        let remote = RemoteUrl::from_request_parts(&mut parts, &())
            .await
            .unwrap();
        assert_eq!(remote.0.as_str(), "https://api.example.com/thing?x=2");

        // No scheme; use Host + X-Forwarded-Proto
        let req = Request::builder()
            .uri("/local/path?y=3")
            .header("host", "example.org")
            .header("X-Forwarded-Proto", "https")
            .body(())
            .unwrap();
        let (mut parts, _) = req.into_parts();
        let remote = RemoteUrl::from_request_parts(&mut parts, &())
            .await
            .unwrap();
        assert_eq!(remote.0.as_str(), "https://example.org/local/path?y=3");
    }
}
