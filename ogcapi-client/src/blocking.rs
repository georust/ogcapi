use std::cell::OnceCell;

use reqwest::{
    Url,
    blocking::Client as ReqwestClient,
    header::{HeaderMap, HeaderValue, USER_AGENT},
};

#[cfg(not(feature = "stac"))]
use ogcapi_types::features::Feature;
#[cfg(feature = "stac")]
use ogcapi_types::{
    common::link_rel::{CHILD, ITEM, SELF},
    stac::{Catalog, Item as Feature, SearchParams, StacEntity},
};
use ogcapi_types::{
    common::{
        Collection, Conformance, LandingPage, Link,
        link_rel::{CONFORMANCE, DATA, NEXT},
    },
    features::FeatureCollection,
};

use crate::Error;

/// Blocking client to access OGC APIs and/or SpatioTemporal Asset Catalogs (STAC).
///
/// # Example
///
/// ```rust,ignore
/// use ogcapi_client::BlockingClient;
///
/// let client = BlockingClient::new("https://example.com/ogc/").unwrap();
/// client.collections().unwrap().for_each(|c| match c {
///     Ok(c) => println!("{}", c.id),
///     Err(e) => eprintln!("{}", e),
/// });
/// ```
#[derive(Clone)]
pub struct BlockingClient {
    pub(crate) client: ReqwestClient,
    pub(crate) endpoint: Url,
    root: OnceCell<LandingPage>,
}

impl BlockingClient {
    /// Creates a BlockingClient for a given `OGCAPI`/`STAC` endpoint.
    pub fn new(endpoint: &str) -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(crate::UA_STRING));

        let client = ReqwestClient::builder()
            .default_headers(headers)
            .build()
            .expect("Build a client");

        Self::new_with(endpoint, client)
    }

    /// Creates a BlockingClient with a custom `reqwest::blocking::Client`.
    pub fn new_with(endpoint: &str, client: ReqwestClient) -> Result<Self, Error> {
        let endpoint = if endpoint.ends_with('/') {
            endpoint.parse::<Url>()?
        } else {
            format!("{endpoint}/").parse::<Url>()?
        };

        Ok(Self {
            client,
            endpoint,
            root: OnceCell::new(),
        })
    }

    /// Returns the landing page or the root catalog.
    pub fn root(&self) -> Result<LandingPage, Error> {
        let root = if let Some(root) = self.root.get() {
            root
        } else {
            let root = self.fetch::<LandingPage>(self.endpoint.as_ref())?;
            self.root.get_or_init(|| root)
        };
        Ok(root.clone())
    }

    /// Returns the conformance declaration of the SpatioTemporal Asset Catalog.
    pub fn conformance(&self) -> Result<Conformance, Error> {
        let catalog = self.root()?;

        #[cfg(feature = "stac")]
        if !catalog.conforms_to.is_empty() {
            return Ok(Conformance {
                conforms_to: catalog.conforms_to,
            });
        }

        if let Some(link) = catalog.links.iter().find(|l| l.rel == CONFORMANCE) {
            return self.fetch::<Conformance>(&link.href);
        }

        Err(Error::ServerError(
            "Unable to retrieve conformance.".to_string(),
        ))
    }

    /// Returns an iterator over the catalogs of the SpatioTemporal Asset Catalog.
    #[cfg(feature = "stac")]
    pub fn catalogs(&self) -> Result<Catalogs, Error> {
        let link = Link::new(&self.endpoint, SELF);
        Ok(Catalogs {
            client: self.to_owned(),
            links: vec![link],
        })
    }

    /// Returns an iterator over the collections.
    pub fn collections(&self) -> Result<Collections, Error> {
        if let Some(link) = self.root()?.links.iter().find(|l| l.rel == DATA) {
            self.fetch::<ogcapi_types::common::Collections>(&link.href)
                .map(|c| Collections {
                    client: self.to_owned(),
                    collections: c.collections.into_iter(),
                    links: c.links,
                })
        } else {
            Err(Error::ClientError(
                "No link found with relation `data`!".to_string(),
            ))
        }
    }

    pub fn collection(&self, id: &str) -> Result<Collection, Error> {
        let url = self.endpoint.join(&format!("collections/{id}"))?;

        self.fetch::<Collection>(url.as_str())
    }

    pub fn items(&self, id: &str) -> Result<Items, Error> {
        let url = self.endpoint.join(&format!("collections/{id}/items"))?;

        self.fetch::<FeatureCollection>(url.as_str())
            .map(|i| Items {
                client: self.to_owned(),
                items: i.features.into_iter(),
                links: i.links,
            })
    }

    /// Returns an iterator over items in a collection, filtered by the given
    /// query parameters (bbox, datetime, limit, etc.).
    pub fn items_with_query(
        &self,
        id: &str,
        query: &ogcapi_types::features::Query,
    ) -> Result<Items, Error> {
        let base = self.endpoint.join(&format!("collections/{id}/items"))?;
        let qs = serde_qs::to_string(query)?;
        let url = if qs.is_empty() {
            base.to_string()
        } else {
            format!("{base}?{qs}")
        };

        self.fetch::<FeatureCollection>(&url).map(|i| Items {
            client: self.to_owned(),
            items: i.features.into_iter(),
            links: i.links,
        })
    }

    /// Returns an iterator over the catalogs of the SpatioTemporal Asset Catalog.
    #[cfg(feature = "stac")]
    pub fn walk(&self) -> Result<StacEntities, Error> {
        let link = Link::new(self.endpoint.to_string(), SELF);
        Ok(StacEntities {
            client: self.to_owned(),
            links: vec![link],
        })
    }

    #[cfg(feature = "stac")]
    pub fn search(&self, params: SearchParams) -> Result<Items, Error> {
        let url = format!("{}search?{}", self.endpoint, serde_qs::to_string(&params)?);

        self.fetch::<FeatureCollection>(&url).map(|i| Items {
            client: self.to_owned(),
            items: i.features.into_iter(),
            links: i.links,
        })
    }

    fn fetch<T>(&self, url: &str) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        log::debug!("Fetching {url}");

        Ok(self
            .client
            .get(url)
            .send()
            .and_then(|rsp| rsp.error_for_status())
            .and_then(|rsp| rsp.json::<T>())?)
    }
}

// --- Processes ---

#[cfg(feature = "processes")]
pub mod processes {
    use ogcapi_types::processes::{Response, Results, StatusInfo, TransmissionMode};

    use crate::{BlockingClient, Error, client::processes::ProcessResponseBody};

    impl BlockingClient {
        pub fn execute(
            &self,
            process_id: &str,
            execute: &ogcapi_types::processes::Execute,
        ) -> Result<ProcessResponseBody, Error> {
            let url = format!("{}processes/{}/execution", self.endpoint, process_id);

            let response = self
                .client
                .post(url)
                .json(execute)
                .send()
                .and_then(|rsp| rsp.error_for_status())?;

            match response.status().as_u16() {
                200 => match execute.response {
                    Response::Raw => {
                        if execute.outputs.len() == 1 {
                            let (_k, v) = execute.outputs.iter().next().unwrap();
                            match v.transmission_mode {
                                TransmissionMode::Value => Ok(ProcessResponseBody::Requested {
                                    outputs: execute.outputs.clone(),
                                    parts: vec![response.bytes()?.to_vec()],
                                }),
                                TransmissionMode::Reference => todo!(),
                            }
                        } else {
                            unimplemented!()
                        }
                    }
                    Response::Document => {
                        Ok(ProcessResponseBody::Results(response.json::<Results>()?))
                    }
                },
                201 => Ok(ProcessResponseBody::StatusInfo(
                    response.json::<StatusInfo>()?,
                )),
                204 => match response.headers().get("link").and_then(|l| l.to_str().ok()) {
                    Some(s) => Ok(ProcessResponseBody::Empty(s.to_string())),
                    None => Err(Error::ServerError(
                        "Missing or malformed `link` header for 204 status response.".to_string(),
                    )),
                },
                _ => Err(Error::ServerError(
                    "Unspecified success status code.".to_string(),
                )),
            }
        }
    }
}

// --- Iterator types ---

#[cfg(feature = "stac")]
pub struct StacEntities {
    client: BlockingClient,
    links: Vec<Link>,
}

#[cfg(feature = "stac")]
pub struct Catalogs {
    client: BlockingClient,
    links: Vec<Link>,
}

pub struct Collections {
    client: BlockingClient,
    collections: <Vec<Collection> as IntoIterator>::IntoIter,
    links: Vec<Link>,
}

pub struct Items {
    client: BlockingClient,
    items: <Vec<Feature> as IntoIterator>::IntoIter,
    links: Vec<Link>,
}

trait Pagination<T> {
    fn try_next(&mut self) -> Result<Option<T>, Error>;
}

#[cfg(feature = "stac")]
impl Pagination<StacEntity> for StacEntities {
    fn try_next(&mut self) -> Result<Option<StacEntity>, Error> {
        if let Some(link) = self.links.pop() {
            let entity = self.client.fetch::<serde_json::Value>(&link.href)?;

            match entity.get("type").and_then(|v| v.as_str()) {
                Some("Catalog") => {
                    let mut catalog = serde_json::from_value::<Catalog>(entity.clone())?;

                    resolve_relative_links(&mut catalog.links, &link.href);

                    let mut children = catalog
                        .links
                        .iter()
                        .filter(|l| l.rel == CHILD || l.rel == ITEM)
                        .cloned()
                        .collect();

                    self.links.append(&mut children);

                    return Ok(Some(StacEntity::Catalog(Box::new(catalog))));
                }
                Some("Collection") => {
                    let mut collection = serde_json::from_value::<Collection>(entity.clone())?;

                    resolve_relative_links(&mut collection.links, &link.href);

                    let mut children = collection
                        .links
                        .iter()
                        .filter(|l| l.rel == CHILD || l.rel == ITEM)
                        .cloned()
                        .collect();

                    self.links.append(&mut children);

                    return Ok(Some(StacEntity::Collection(Box::new(collection))));
                }
                Some("Feature") => {
                    let mut item = serde_json::from_value::<Feature>(entity.clone())?;

                    resolve_relative_links(&mut item.links, &link.href);

                    let mut children = item
                        .links
                        .iter()
                        .filter(|l| l.rel == CHILD || l.rel == ITEM)
                        .cloned()
                        .collect();

                    self.links.append(&mut children);

                    return Ok(Some(StacEntity::Item(Box::new(item))));
                }
                _ => return Err(Error::ClientError("Unknown STAC entity!".to_string())),
            };
        }
        Ok(None)
    }
}

#[cfg(feature = "stac")]
impl Pagination<Catalog> for Catalogs {
    fn try_next(&mut self) -> Result<Option<Catalog>, Error> {
        while let Some(link) = self.links.pop() {
            let mut catalog = self.client.fetch::<Catalog>(&link.href)?;

            if catalog.r#type == "Catalog" {
                resolve_relative_links(&mut catalog.links, &link.href);

                let mut children = catalog
                    .links
                    .iter()
                    .filter(|l| l.rel == CHILD)
                    .cloned()
                    .collect();

                self.links.append(&mut children);

                return Ok(Some(catalog));
            } else {
                continue;
            }
        }
        Ok(None)
    }
}

impl Pagination<Collection> for Collections {
    fn try_next(&mut self) -> Result<Option<Collection>, Error> {
        if let Some(value) = self.collections.next() {
            return Ok(Some(value));
        }

        if let Some(link) = self.links.iter().find(|l| l.rel == NEXT) {
            match self
                .client
                .fetch::<ogcapi_types::common::Collections>(&link.href)
            {
                Ok(c) => {
                    self.collections = c.collections.into_iter();
                    self.links = c.links;
                    if let Some(value) = self.collections.next() {
                        Ok(Some(value))
                    } else {
                        Ok(None)
                    }
                }
                Err(err) => {
                    // reset links to prevent loop
                    self.links = Vec::new();
                    Err(err)
                }
            }
        } else {
            Ok(None)
        }
    }
}

impl Pagination<Feature> for Items {
    fn try_next(&mut self) -> Result<Option<Feature>, Error> {
        if let Some(value) = self.items.next() {
            return Ok(Some(value));
        }

        if let Some(link) = self.links.iter().find(|l| l.rel == NEXT) {
            match self.client.fetch::<FeatureCollection>(&link.href) {
                Ok(i) => {
                    self.items = i.features.into_iter();
                    self.links = i.links;
                    if let Some(value) = self.items.next() {
                        Ok(Some(value))
                    } else {
                        Ok(None)
                    }
                }
                Err(err) => {
                    // reset links to prevent loop
                    self.links = Vec::new();
                    Err(err)
                }
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(feature = "stac")]
impl Iterator for StacEntities {
    type Item = Result<StacEntity, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(value)) => Some(Ok(value)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

impl Iterator for Collections {
    type Item = Result<Collection, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(value)) => Some(Ok(value)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

impl Iterator for Items {
    type Item = Result<Feature, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(value)) => Some(Ok(value)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

#[cfg(feature = "stac")]
impl Iterator for Catalogs {
    type Item = Result<Catalog, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(value)) => Some(Ok(value)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

#[cfg(feature = "stac")]
fn resolve_relative_links(links: &mut [Link], base: &str) {
    let base_url = Url::parse(base).expect("Parse base url from string");

    links.iter_mut().for_each(|l| match Url::parse(&l.href) {
        Ok(_) => (),
        Err(url::ParseError::RelativeUrlWithoutBase) => {
            l.href = base_url.join(&l.href).unwrap().to_string();
        }
        Err(e) => log::error!("{e}"),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "stac")]
    fn version() {
        let endpoint = "https://data.geo.admin.ch/api/stac/v0.9/";
        let client = BlockingClient::new(endpoint).unwrap();
        assert_eq!("0.9.0", client.root().unwrap().stac_version);
    }

    #[test]
    fn conformance() {
        let endpoint = "https://data.geo.admin.ch/api/stac/v0.9/";
        let client = BlockingClient::new(endpoint).unwrap();
        let conformance = client.conformance().unwrap();
        println!("{conformance:#?}");
        assert!(!conformance.conforms_to.is_empty());
    }

    #[test]
    fn collection() {
        let endpoint = "https://data.geo.admin.ch/api/stac/v0.9/";
        let client = BlockingClient::new(endpoint).unwrap();
        let collection = client.collection("ch.swisstopo.swissalti3d").unwrap();
        assert_eq!("ch.swisstopo.swissalti3d", collection.id);
    }

    #[test]
    fn collections() {
        let endpoint = "https://data.geo.admin.ch/api/stac/v0.9/";
        let client = BlockingClient::new(endpoint).unwrap();
        let collections = client
            .collections()
            .unwrap()
            .collect::<Vec<Result<ogcapi_types::common::Collection, crate::Error>>>();
        for collection in &collections {
            if let Ok(c) = collection.as_ref() {
                println!("{}", c.id)
            };
        }
        assert!(!collections.is_empty());
    }

    #[test]
    #[cfg(feature = "stac")]
    fn search() {
        let endpoint = "https://data.geo.admin.ch/api/stac/v0.9/";
        let client = BlockingClient::new(endpoint).unwrap();
        let bbox = ogcapi_types::common::Bbox::from([7.4473, 46.9479, 7.4475, 46.9481]);
        let params = ogcapi_types::stac::SearchParams::new()
            .with_bbox(bbox)
            .with_collections(["ch.swisstopo.swissalti3d"]);
        let mut items = client.search(params).unwrap();
        let item = items.next().unwrap().unwrap();
        assert_eq!("swissalti3d_2019_2600-1199", item.id.unwrap().to_string());
        assert_eq!(
            Some("ch.swisstopo.swissalti3d".to_string()),
            item.collection
        );
    }
}
