use std::future::Future;
use std::pin::Pin;

use reqwest::{
    Client as ReqwestClient, StatusCode, Url,
    header::{HeaderMap, HeaderValue, LOCATION, USER_AGENT},
};

use ogcapi_types::common::{Conformance, LandingPage, link_rel::CONFORMANCE};

use crate::Error;

pub mod collections;

#[cfg(feature = "features")]
pub mod features;

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
    pub fn new(endpoint: impl ToString) -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(crate::UA_STRING));

        let client = ReqwestClient::builder()
            .default_headers(headers)
            .build()
            .expect("Build a client");

        Self::new_with(endpoint, client)
    }

    /// Creates a Client with a custom `reqwest::Client`.
    pub fn new_with(endpoint: impl ToString, client: ReqwestClient) -> Result<Self, Error> {
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

        Err(Error::UnknownConformance(
            "Unable to retrieve conformance.".to_string(),
        ))
    }

    /// Fetch a JSON resource.
    pub async fn fetch<T>(&self, url: &str) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        log::debug!("Fetching {url}");
        // TODO: extract and propagate exception bodies
        self.client
            .get(url)
            .send()
            .await
            .and_then(|rsp| rsp.error_for_status())
            .map_err(Error::RequestError)?
            .json::<T>()
            .await
            .map_err(Error::RequestError)
    }

    /// Create a JSON resource returning the `Location` header.
    pub async fn post<T>(&self, url: &str, data: &T) -> Result<String, Error>
    where
        T: serde::Serialize,
    {
        let response = self
            .client
            .post(url)
            .json(data)
            .send()
            .await
            .map_err(Error::RequestError)?
            .error_for_status()
            .map_err(Error::RequestError)?;
        // TODO: extract and propagate exception bodies
        debug_assert_eq!(response.status(), StatusCode::CREATED);
        debug_assert!(response.headers().contains_key(LOCATION));
        Ok(response.headers()["Location"]
            .to_str()
            .map_err(|e| Error::ClientError(e.to_string()))?
            .to_owned())
    }

    /// Update a JSON resource.
    pub async fn put<T>(&self, url: &str, data: &T) -> Result<(), Error>
    where
        T: serde::Serialize,
    {
        let response = self
            .client
            .put(url)
            .json(data)
            .send()
            .await
            .map_err(Error::RequestError)?
            .error_for_status()
            .map_err(Error::RequestError)?;
        // TODO: extract and propagate exception bodies
        debug_assert_eq!(response.status(), StatusCode::NO_CONTENT);
        Ok(())
    }

    /// Delete a resource.
    pub async fn delete(&self, url: &str) -> Result<(), Error> {
        let response = self
            .client
            .delete(url)
            .send()
            .await
            .map_err(Error::RequestError)?
            .error_for_status()
            .map_err(Error::RequestError)?;
        // TODO: extract and propagate exception bodies
        debug_assert_eq!(response.status(), StatusCode::NO_CONTENT);
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn conformance() {
        let endpoint = "https://data.geo.admin.ch/api/stac/v0.9/";
        let client = Client::new(endpoint).unwrap();
        let conformance = client.conformance().await.unwrap();
        println!("{conformance:#?}");
        assert!(!conformance.conforms_to.is_empty());
    }
}
