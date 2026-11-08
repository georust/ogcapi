use std::future::Future;
use std::pin::Pin;

use reqwest::{
    Client as ReqwestClient, RequestBuilder, Response, Url,
    header::{CONTENT_TYPE, HeaderMap, HeaderValue, LOCATION, USER_AGENT},
};

use ogcapi_types::common::{
    Conformance, Exception, LandingPage, link_rel::CONFORMANCE, media_type::PROBLEM_JSON,
};

use crate::Error;

pub mod collections;

#[cfg(feature = "features")]
pub mod features;

#[cfg(feature = "processes")]
pub mod processes;

type BoxFuture<T> = Pin<Box<dyn Future<Output = Result<T, Error>>>>;

/// Async client to access OGC APIs and/or SpatioTemporal Asset Catalogs (STAC).
///
/// # Example
///
/// ```rust,ignore
/// use ogcapi_client::Client;
///
/// #[tokio::main]
/// async fn main() {
///     let client = Client::new("https://example.com/ogc/").unwrap();
///     let collection = client.collection("my-collection").await.unwrap();
/// }
/// ```
#[derive(Clone)]
pub struct Client {
    pub(crate) client: ReqwestClient,
    pub(crate) endpoint: Url,
}

impl Client {
    /// Creates a Client for a given OGC API endpoint.
    pub fn new(endpoint: &str) -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(crate::UA_STRING));

        let client = ReqwestClient::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| Error::ClientError(e.to_string()))?;

        Self::new_with(endpoint, client)
    }

    /// Creates a Client with a custom `reqwest::Client`.
    pub fn new_with(endpoint: &str, client: ReqwestClient) -> Result<Self, Error> {
        let endpoint = endpoint.to_string();
        let endpoint = if endpoint.ends_with('/') {
            Url::parse(&endpoint)?
        } else {
            format!("{endpoint}/").parse::<Url>()?
        };

        Ok(Self { client, endpoint })
    }

    /// Returns the landing page.
    pub async fn root(&self) -> Result<LandingPage, Error> {
        self.fetch::<LandingPage>(self.endpoint.as_ref()).await
    }

    /// Returns the conformance declaration.
    pub async fn conformance(&self) -> Result<Conformance, Error> {
        let root = self.root().await?;

        #[cfg(feature = "stac")]
        if !root.conforms_to.is_empty() {
            return Ok(Conformance {
                conforms_to: root.conforms_to,
            });
        }

        if let Some(link) = root.links.iter().find(|l| l.rel == CONFORMANCE) {
            return self.fetch::<Conformance>(&link.href).await;
        }

        Err(Error::ServerError(
            "Unable to retrieve conformance.".to_string(),
        ))
    }

    /// Fetch a JSON resource.
    pub async fn fetch<T>(&self, url: &str) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        log::debug!("Fetching {url}");
        json(self.client.get(url)).await
    }

    /// Create a JSON resource returning the `Location` header.
    pub async fn post<T>(&self, url: &str, data: &T) -> Result<String, Error>
    where
        T: serde::Serialize,
    {
        let response = send(self.client.post(url).json(data)).await?;
        let location = response
            .headers()
            .get(LOCATION)
            .ok_or_else(|| Error::ServerError(format!("{url} answered no Location")))?;
        Ok(location
            .to_str()
            .map_err(|e| Error::ClientError(e.to_string()))?
            .to_owned())
    }

    /// Update a JSON resource.
    pub async fn put<T>(&self, url: &str, data: &T) -> Result<(), Error>
    where
        T: serde::Serialize,
    {
        send(self.client.put(url).json(data)).await?;
        Ok(())
    }

    /// Delete a resource.
    pub async fn delete(&self, url: &str) -> Result<(), Error> {
        send(self.client.delete(url)).await?;
        Ok(())
    }

    /// The URL of `segments` below the endpoint.
    pub(crate) fn url(&self, segments: &[&str]) -> Result<Url, Error> {
        url_below(&self.endpoint, segments)
    }
}

/// The URL of `segments` below `endpoint`, each a path segment of its own (an
/// id holding `/`, `?` or `#` addresses no other resource); a refusal of a
/// segment that is empty, `.` or `..`.
pub(crate) fn url_below(endpoint: &Url, segments: &[&str]) -> Result<Url, Error> {
    if let Some(segment) = segments.iter().find(|s| matches!(**s, "" | "." | "..")) {
        return Err(Error::ClientError(format!("`{segment}` is no id")));
    }
    let mut url = endpoint.clone();
    url.path_segments_mut()
        .map_err(|()| Error::ClientError(format!("{endpoint} has no path")))?
        .pop_if_empty()
        .extend(segments);
    Ok(url)
}

/// The answer to `request`, an error for an error status: the server's
/// exception where it answers a problem document.
pub(crate) async fn send(request: RequestBuilder) -> Result<Response, Error> {
    let response = extract_exception(request.send().await?).await?;
    Ok(response.error_for_status()?)
}

/// The JSON answer to `request`.
pub(crate) async fn json<T: serde::de::DeserializeOwned>(
    request: RequestBuilder,
) -> Result<T, Error> {
    Ok(send(request).await?.json::<T>().await?)
}

/// The exception of a response that is a problem document (RFC 9457), with
/// its status where the document leaves it out (section 3.1.2).
pub(crate) async fn extract_exception(response: Response) -> Result<Response, Error> {
    let problem = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|content_type| content_type.to_str().ok())
        .and_then(|content_type| content_type.split(';').next())
        .is_some_and(|essence| essence.trim().eq_ignore_ascii_case(PROBLEM_JSON));
    if problem {
        let status = response.status().as_u16();
        let mut exception = response.json::<Exception>().await?;
        exception.status.get_or_insert(status);
        Err(Error::ExceptionError(Box::new(exception)))
    } else {
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn an_id_is_a_path_segment_of_its_own() {
        let endpoint = super::Url::parse("http://example.org/api/").unwrap();
        let url = super::url_below(&endpoint, &["collections", "a/b?c#d", "items"]).unwrap();
        assert_eq!(
            url.as_str(),
            "http://example.org/api/collections/a%2Fb%3Fc%23d/items"
        );
        assert!(super::url_below(&endpoint, &["collections", ".."]).is_err());
        assert!(super::url_below(&endpoint, &["collections", ""]).is_err());
    }

    use super::*;

    #[tokio::test]
    #[ignore = "reaches data.geo.admin.ch"]
    async fn conformance() {
        let endpoint = "https://data.geo.admin.ch/api/stac/v0.9/";
        let client = Client::new(endpoint).unwrap();
        let conformance = client.conformance().await.unwrap();
        assert_ne!(conformance.conforms_to, [] as [String; 0]);
    }
}
