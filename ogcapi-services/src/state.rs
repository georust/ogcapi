use std::{collections::BTreeMap, sync::RwLock};

#[cfg(feature = "edr")]
use ogcapi_drivers::EdrQuerier;
#[cfg(feature = "features")]
use ogcapi_drivers::FeatureTransactions;
#[cfg(feature = "processes")]
use ogcapi_drivers::JobHandler;
#[cfg(feature = "styles")]
use ogcapi_drivers::StyleTransactions;
#[cfg(feature = "tiles")]
use ogcapi_drivers::TileTransactions;

use ogcapi_drivers::{postgres::Db, CollectionTransactions};
use ogcapi_types::common::{
    link_rel::{CONFORMANCE, SELF, SERVICE_DESC},
    media_type::{JSON, OPEN_API_JSON},
    Conformance, LandingPage, Link,
};

use crate::Processor;

// #[derive(Clone)]
pub struct State {
    pub drivers: Drivers,
    pub root: RwLock<LandingPage>,
    pub conformance: RwLock<Conformance>,
    pub openapi: openapiv3::OpenAPI,
    #[cfg(feature = "processes")]
    pub processors: BTreeMap<String, Box<dyn Processor>>,
}

// TODO: Introduce service trait
pub struct Drivers {
    pub collections: Box<dyn CollectionTransactions>,
    #[cfg(feature = "features")]
    pub features: Box<dyn FeatureTransactions>,
    #[cfg(feature = "edr")]
    pub edr: Box<dyn EdrQuerier>,
    #[cfg(feature = "processes")]
    pub jobs: Box<dyn JobHandler>,
    #[cfg(feature = "styles")]
    pub styles: Box<dyn StyleTransactions>,
    #[cfg(feature = "tiles")]
    pub tiles: Box<dyn TileTransactions>,
}

impl State {
    pub fn new(db: Db, api: &[u8]) -> Self {
        let openapi: openapiv3::OpenAPI = serde_yaml::from_slice(api).unwrap();

        let root = RwLock::new(LandingPage {
            #[cfg(feature = "stac")]
            id: "root".to_string(),
            title: Some(openapi.info.title.to_owned()),
            description: openapi.info.description.to_owned(),
            links: vec![
                Link::new(".", SELF).title("This document").mediatype(JSON),
                Link::new("api", SERVICE_DESC)
                    .title("The Open API definition")
                    .mediatype(OPEN_API_JSON),
                Link::new("conformance", CONFORMANCE)
                    .title("OGC conformance classes implemented by this API")
                    .mediatype(JSON),
            ],
            ..Default::default()
        });

        let conformance = RwLock::new(Conformance {
            conforms_to: vec![
                "http://www.opengis.net/spec/ogcapi-common-1/1.0/req/core".to_string(),
                "http://www.opengis.net/spec/ogcapi-common-2/1.0/req/collections".to_string(),
                "http://www.opengis.net/spec/ogcapi_common-2/1.0/req/json".to_string(),
            ],
        });

        let drivers = Drivers {
            collections: Box::new(db.clone()),
            #[cfg(feature = "features")]
            features: Box::new(db.clone()),
            #[cfg(feature = "edr")]
            edr: Box::new(db.clone()),
            #[cfg(feature = "processes")]
            jobs: Box::new(db.clone()),
            #[cfg(feature = "styles")]
            styles: Box::new(db.clone()),
            #[cfg(feature = "tiles")]
            tiles: Box::new(db),
        };

        State {
            drivers,
            root,
            conformance,
            openapi,
            #[cfg(feature = "processes")]
            processors: Default::default(),
        }
    }

    pub fn register_processes(&mut self, processors: Vec<Box<dyn Processor>>) {
        for p in processors {
            self.processors.insert(p.id(), p);
        }
    }
}
