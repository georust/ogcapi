use reqwest::{
    Client as ReqwestClient, Url,
    header::{HeaderMap, HeaderValue, USER_AGENT},
};

use ogcapi_types::common::{
    Collection, Conformance, LandingPage,
    link_rel::{CONFORMANCE, DATA, NEXT},
};
use ogcapi_types::features::FeatureCollection;
#[cfg(feature = "stac")]
use ogcapi_types::stac::SearchParams;

use crate::Error;

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
            .expect("Build a client");

        Self::new_with(endpoint, client)
    }

    /// Creates a Client with a custom `reqwest::Client`.
    pub fn new_with(endpoint: &str, client: ReqwestClient) -> Result<Self, Error> {
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

    /// Searches items with the given parameters.
    #[cfg(feature = "stac")]
    pub async fn search(&self, params: SearchParams) -> Result<FeatureCollection, Error> {
        let url = format!("{}search?{}", self.endpoint, serde_qs::to_string(&params)?);
        self.fetch::<FeatureCollection>(&url).await
    }

    /// Fetches the next page of results by following the `next` link.
    /// Returns `None` if there is no next link.
    pub async fn next_page<T>(
        &self,
        links: &[ogcapi_types::common::Link],
    ) -> Result<Option<T>, Error>
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

    /// Fetches all collections, following pagination links.
    pub async fn all_collections(&self) -> Result<Vec<Collection>, Error> {
        let mut all_collections = Vec::new();
        let mut page = self.collections().await?;

        loop {
            all_collections.extend(page.collections);

            match self
                .next_page::<ogcapi_types::common::Collections>(&page.links)
                .await?
            {
                Some(next) => page = next,
                None => break,
            }
        }

        Ok(all_collections)
    }

    pub(crate) async fn fetch<T>(&self, url: &str) -> Result<T, Error>
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

    #[tokio::test]
    async fn collection() {
        let endpoint = "https://data.geo.admin.ch/api/stac/v0.9/";
        let client = Client::new(endpoint).unwrap();
        let collection = client.collection("ch.swisstopo.swissalti3d").await.unwrap();
        assert_eq!("ch.swisstopo.swissalti3d", collection.id);
    }

    #[tokio::test]
    async fn collections() {
        let endpoint = "https://data.geo.admin.ch/api/stac/v0.9/";
        let client = Client::new(endpoint).unwrap();
        let collections = client.all_collections().await.unwrap();
        for c in &collections {
            println!("{}", c.id);
        }
        assert!(!collections.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "stac")]
    async fn search() {
        let endpoint = "https://data.geo.admin.ch/api/stac/v0.9/";
        let client = Client::new(endpoint).unwrap();
        let bbox = ogcapi_types::common::Bbox::from([7.4473, 46.9479, 7.4475, 46.9481]);
        let params = ogcapi_types::stac::SearchParams::new()
            .with_bbox(bbox)
            .with_collections(["ch.swisstopo.swissalti3d"]);
        let result = client.search(params).await.unwrap();
        let item = &result.features[0];
        assert_eq!(
            Some("ch.swisstopo.swissalti3d".to_string()),
            item.collection.clone()
        );
    }
}
