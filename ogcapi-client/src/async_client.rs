use reqwest::{
    Client as ReqwestClient, Url,
    header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT},
};

use ogcapi_types::common::{
    Collection, Conformance, LandingPage,
    link_rel::{DATA, NEXT},
};
use ogcapi_types::features::FeatureCollection;

use crate::Error;

static UA_STRING: &str = "OGCAPI-CLIENT";

/// Async client to access OGC APIs, compatible with both native and WASM targets.
///
/// # Example
///
/// ```rust,ignore
/// use ogcapi_client::AsyncClient;
///
/// let client = AsyncClient::new("https://example.com/ogc/").unwrap();
/// let collection = client.collection("my-collection").await.unwrap();
/// ```
#[derive(Clone)]
pub struct AsyncClient {
    client: ReqwestClient,
    endpoint: Url,
}

impl AsyncClient {
    /// Creates an AsyncClient for a given OGC API endpoint.
    pub fn new(endpoint: &str) -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(UA_STRING));

        let client = ReqwestClient::builder()
            .default_headers(headers)
            .build()
            .expect("Build a client");

        let endpoint = if endpoint.ends_with('/') {
            endpoint.parse::<Url>()?
        } else {
            format!("{endpoint}/").parse::<Url>()?
        };

        Ok(Self { client, endpoint })
    }

    /// Creates an AsyncClient with bearer token authentication.
    pub fn with_bearer_token(endpoint: &str, token: &str) -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(UA_STRING));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|e| Error::ClientError(format!("Invalid bearer token: {e}")))?,
        );

        let client = ReqwestClient::builder()
            .default_headers(headers)
            .build()
            .expect("Build a client");

        let endpoint = if endpoint.ends_with('/') {
            endpoint.parse::<Url>()?
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

        if let Some(link) = root.links.iter().find(|l| l.rel == "conformance") {
            return self.fetch::<Conformance>(&link.href).await;
        }

        Err(Error::UnknownConformance(
            "Unable to retrieve conformance.".to_string(),
        ))
    }

    /// Returns all collections (first page).
    pub async fn collections(&self) -> Result<ogcapi_types::common::Collections, Error> {
        let root = self.root().await?;

        if let Some(link) = root.links.iter().find(|l| l.rel == DATA) {
            self.fetch::<ogcapi_types::common::Collections>(&link.href)
                .await
        } else {
            Err(Error::ClientError(
                "No link found with relation `data`!".to_string(),
            ))
        }
    }

    /// Returns a single collection by id.
    pub async fn collection(&self, id: &str) -> Result<Collection, Error> {
        let url = self.endpoint.join(&format!("collections/{id}"))?;
        self.fetch::<Collection>(url.as_str()).await
    }

    /// Returns items for a collection (first page).
    pub async fn items(&self, id: &str) -> Result<FeatureCollection, Error> {
        let url = self.endpoint.join(&format!("collections/{id}/items"))?;
        self.fetch::<FeatureCollection>(url.as_str()).await
    }

    /// Fetches the next page of results by following the `next` link.
    /// Returns `None` if there is no next link.
    pub async fn next_page<T>(&self, links: &[ogcapi_types::common::Link]) -> Result<Option<T>, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        if let Some(link) = links.iter().find(|l| l.rel == NEXT) {
            Ok(Some(self.fetch::<T>(&link.href).await?))
        } else {
            Ok(None)
        }
    }

    /// Fetches all items for a collection, following pagination links.
    pub async fn all_items(&self, id: &str) -> Result<Vec<ogcapi_types::features::Feature>, Error> {
        let mut all_features = Vec::new();
        let mut page = self.items(id).await?;

        loop {
            all_features.extend(page.features);

            match self.next_page::<FeatureCollection>(&page.links).await? {
                Some(next) => page = next,
                None => break,
            }
        }

        Ok(all_features)
    }

    async fn fetch<T>(&self, url: &str) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        log::debug!("Fetching {url}");

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
}
