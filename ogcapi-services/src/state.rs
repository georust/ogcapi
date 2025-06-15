use std::sync::{Arc, RwLock};

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

use ogcapi_drivers::{CollectionTransactions, postgres::Db};
#[cfg(feature = "processes")]
use ogcapi_processes::Processor;
use ogcapi_types::common::{Conformance, LandingPage};

use crate::{Config, ConfigParser};

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub root: Arc<RwLock<LandingPage>>,
    pub conformance: Arc<RwLock<Conformance>>,
    pub drivers: Arc<Drivers>,
    pub db: Db,
    #[cfg(feature = "stac")]
    pub s3: ogcapi_drivers::s3::S3,
    #[cfg(feature = "processes")]
    pub processors: Arc<RwLock<std::collections::HashMap<String, Box<dyn Processor>>>>,
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

impl AppState {
    pub async fn new() -> Self {
        let config = Config::parse();
        AppState::new_from(&config).await
    }

    pub async fn new_from(config: &Config) -> Self {
        let db = Db::setup(&config.database_url).await.unwrap();
        AppState::new_with(db).await
    }

    pub async fn new_with(db: Db) -> Self {
        // conformance
        #[allow(unused_mut)]
        let mut conformace = Conformance::default();
        #[cfg(feature = "stac")]
        conformace.extend(&[
            "https://api.stacspec.org/v1.0.0-rc.1/core",
            "https://api.stacspec.org/v1.0.0-rc.1/item-search",
            "https://api.stacspec.org/v1.0.0-rc.1/collections",
            "https://api.stacspec.org/v1.0.0-rc.1/ogcapi-features",
            "https://api.stacspec.org/v1.0.0-rc.1/browseable",
        ]);

        // drivers
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
            tiles: Box::new(db.clone()),
        };

        AppState {
            root: Arc::new(RwLock::new(LandingPage::new("root").description("root"))),
            conformance: Arc::new(RwLock::new(conformace)),
            drivers: Arc::new(drivers),
            db,
            #[cfg(feature = "stac")]
            s3: ogcapi_drivers::s3::S3::new().await,
            #[cfg(feature = "processes")]
            processors: Default::default(),
        }
    }

    pub fn root(mut self, root: LandingPage) -> Self {
        self.root = Arc::new(RwLock::new(root));
        self
    }

    #[cfg(feature = "stac")]
    pub async fn s3_client(mut self, client: ogcapi_drivers::s3::S3) -> Self {
        self.s3 = client;
        self
    }

    #[cfg(feature = "processes")]
    pub fn processors(self, processors: Vec<Box<dyn Processor>>) -> Self {
        for p in processors {
            self.processors
                .write()
                .unwrap()
                .insert(p.id().to_string(), p);
        }
        self
    }
}
