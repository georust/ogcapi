#[cfg(feature = "processes")]
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[cfg(feature = "edr")]
use ogcapi_drivers::EdrQuerier;
#[cfg(feature = "features")]
use ogcapi_drivers::FeatureTransactions;
#[cfg(feature = "processes")]
use ogcapi_drivers::JobHandler;
#[cfg(feature = "stac")]
use ogcapi_drivers::StacSearch;
#[cfg(feature = "styles")]
use ogcapi_drivers::StyleTransactions;
#[cfg(feature = "tiles")]
use ogcapi_drivers::TileTransactions;

use ogcapi_drivers::{CollectionTransactions, postgres::Db};
#[cfg(feature = "processes")]
use ogcapi_processes::Processor;
use ogcapi_types::common::{Conformance, LandingPage};
use url::Url;
use utoipa::openapi::OpenApi;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub(crate) root: Arc<RwLock<LandingPage>>,
    pub(crate) conformance: Arc<RwLock<Conformance>>,
    pub(crate) openapi: Arc<OpenApi>,
    pub(crate) drivers: Arc<Drivers>,
    #[cfg(feature = "processes")]
    pub(crate) processors: Arc<RwLock<HashMap<String, Box<dyn Processor>>>>,
    #[cfg(feature = "processes")]
    pub(crate) spawn: fn(futures::future::BoxFuture<'static, ()>) -> tokio::task::JoinHandle<()>,
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
    #[cfg(feature = "stac")]
    pub stac: Box<dyn StacSearch>,
}

impl Drivers {
    /// Try to setup drivers from `DATABASE_URL` environment variable.
    pub async fn try_new_from_env() -> Result<Self, anyhow::Error> {
        let var = std::env::var("DATABASE_URL")?;
        Self::try_new_db(&var).await
    }

    /// Try to setup db driver from database url.
    pub async fn try_new_db(url: &str) -> Result<Self, anyhow::Error> {
        let database_url = Url::parse(url)?;
        let db = Db::setup(&database_url).await?;

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
            #[cfg(feature = "stac")]
            stac: Box::new(db.clone()),
        };

        Ok(drivers)
    }
}

impl AppState {
    pub async fn new(drivers: Drivers) -> Self {
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

        AppState {
            root: Arc::new(RwLock::new(LandingPage::new("root").description("root"))),
            conformance: Arc::new(RwLock::new(conformace)),
            openapi: Arc::new(OpenApi::default()),
            drivers: Arc::new(drivers),
            #[cfg(feature = "processes")]
            processors: Default::default(),
            #[cfg(feature = "processes")]
            spawn: tokio::spawn,
        }
    }

    pub fn root(mut self, root: LandingPage) -> Self {
        self.root = Arc::new(RwLock::new(root));
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

    #[cfg(feature = "processes")]
    pub fn with_spawn_fn(
        mut self,
        spawn_fn: fn(futures::future::BoxFuture<'static, ()>) -> tokio::task::JoinHandle<()>,
    ) -> Self {
        self.spawn = spawn_fn;
        self
    }

    /// Set initial [`OpenApi`] document for the service, which is empty by default.
    /// This can be used to add custom paths or components to the [`OpenApi`] document.
    /// The provided [`OpenApi`] document will be merged with the auto-generated one from the [`Service`].
    ///
    /// Note: The provided [`OpenApi`] document should not contain any paths or components that are already defined by the service, as this may lead to conflicts.
    pub fn with_openapi(mut self, openapi: OpenApi) -> Self {
        self.openapi = Arc::new(openapi);
        self
    }
}
