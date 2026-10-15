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
use ogcapi_processes::DynProcessor;
use ogcapi_types::common::{Conformance, LandingPage};
use url::Url;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub(crate) root: Arc<RwLock<LandingPage>>,
    pub(crate) conformance: Arc<RwLock<Conformance>>,
    pub(crate) drivers: Arc<Drivers>,
    #[cfg(feature = "processes")]
    pub(crate) processes: ProcessesState,
}

/// Application state for OGC API Processes
#[cfg(feature = "processes")]
#[derive(Clone)]
pub struct ProcessesState {
    pub(crate) processors: Arc<RwLock<HashMap<String, Arc<dyn DynProcessor>>>>,
    pub(crate) spawn: fn(futures::future::BoxFuture<'static, ()>) -> tokio::task::JoinHandle<()>,
    pub(crate) sync_process_call_is_job: bool,
}

#[cfg(feature = "processes")]
mod process_state {
    use super::*;
    use crate::{Error, Result, util::read_lock};
    use axum::http::StatusCode;

    impl Default for ProcessesState {
        fn default() -> Self {
            Self {
                processors: Arc::new(RwLock::new(HashMap::new())),
                spawn: tokio::spawn,
                sync_process_call_is_job: false,
            }
        }
    }

    #[cfg(feature = "processes")]
    impl ProcessesState {
        pub fn processor_by_id(&self, process_id: &str) -> Result<Arc<dyn DynProcessor>> {
            read_lock(&self.processors)
                .get(process_id)
                .cloned()
                .ok_or_else(|| {
                    Error::ApiException(
                        (
                            StatusCode::NOT_FOUND,
                            format!("No process with id `{process_id}`"),
                        )
                            .into(),
                    )
                })
        }
    }
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
    pub fn new(drivers: Drivers) -> impl Future<Output = Self> {
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

        std::future::ready(AppState {
            root: Arc::new(RwLock::new(LandingPage::new("root").description("root"))),
            conformance: Arc::new(RwLock::new(conformace)),
            drivers: Arc::new(drivers),
            #[cfg(feature = "processes")]
            processes: ProcessesState::default(),
        })
    }

    /// Override the default root landing page for the application.
    #[must_use]
    pub fn root(mut self, root: LandingPage) -> Self {
        self.root = Arc::new(RwLock::new(root));
        self
    }

    /// Configure the processors for the application.
    #[cfg(feature = "processes")]
    #[must_use]
    pub fn processors<P: crate::processes::IntoArcProcessor>(
        self,
        processors: impl IntoIterator<Item = P>,
    ) -> Self {
        crate::util::write_lock(&self.processes.processors).extend(processors.into_iter().map(
            |processor| {
                let processor = processor.into_arc_processor();
                (processor.id().to_string(), processor)
            },
        ));

        self
    }

    /// Configure a custom spawn function for asynchronous tasks.
    /// This allows passing on custom tracing spans or task local variables.
    #[cfg(feature = "processes")]
    #[must_use]
    pub fn with_spawn_fn(
        mut self,
        spawn_fn: fn(futures::future::BoxFuture<'static, ()>) -> tokio::task::JoinHandle<()>,
    ) -> Self {
        self.processes.spawn = spawn_fn;
        self
    }

    /// Configure whether synchronous process calls will create jobs
    /// in the background
    #[cfg(feature = "processes")]
    #[must_use]
    pub fn sync_process_calls_are_jobs(mut self, sync: bool) -> Self {
        self.processes.sync_process_call_is_job = sync;
        self
    }
}
