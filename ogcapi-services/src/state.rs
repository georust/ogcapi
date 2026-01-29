#[cfg(feature = "processes")]
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::{extract::Request, response::Response};
use futures::future::BoxFuture;
#[cfg(feature = "edr")]
use ogcapi_drivers::EdrQuerier;
#[cfg(feature = "features")]
use ogcapi_drivers::FeatureTransactions;
#[cfg(feature = "processes")]
use ogcapi_drivers::JobHandler;
use ogcapi_drivers::NoUser;
#[cfg(feature = "stac")]
use ogcapi_drivers::StacSearch;
#[cfg(feature = "styles")]
use ogcapi_drivers::StyleTransactions;
#[cfg(feature = "tiles")]
use ogcapi_drivers::TileTransactions;

use ogcapi_drivers::{CollectionTransactions, postgres::Db};
#[cfg(feature = "processes")]
use ogcapi_processes::Processor;
use ogcapi_types::common::{Conformance, LandingPage, Link};
use tower_http::auth::AsyncAuthorizeRequest;
use url::Url;

#[cfg(feature = "processes")]
type BoxedProcessor<User> = Box<dyn Processor<User = User>>;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub(crate) root: Arc<RwLock<LandingPage>>,
    pub(crate) conformance: Arc<RwLock<Conformance>>,
    pub(crate) drivers: Arc<Drivers>,
    #[cfg(feature = "processes")]
    pub(crate) processors: Arc<RwLock<HashMap<String, BoxedProcessor<NoUser>>>>,
}

// TODO: Introduce service trait
pub struct Drivers {
    pub collections: Box<dyn CollectionTransactions>,
    #[cfg(feature = "features")]
    pub features: Box<dyn FeatureTransactions>,
    #[cfg(feature = "edr")]
    pub edr: Box<dyn EdrQuerier>,
    #[cfg(feature = "processes")]
    pub jobs: Box<dyn JobHandler<User = NoUser>>,
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
            drivers: Arc::new(drivers),
            #[cfg(feature = "processes")]
            processors: Default::default(),
        }
    }

    pub fn root(mut self, root: LandingPage) -> Self {
        self.root = Arc::new(RwLock::new(root));
        self
    }

    #[cfg(feature = "processes")]
    pub fn processors(self, processors: Vec<Box<dyn Processor<User = NoUser>>>) -> Self {
        for p in processors {
            self.processors
                .write()
                .unwrap()
                .insert(p.id().to_string(), p);
        }
        self
    }
}

pub trait OgcApiState: Send + Sync + Clone + 'static
where
    <Self::AuthLayer as AsyncAuthorizeRequest<axum::body::Body>>::Future: Send,
{
    type User: Send + Sync + Clone;
    type AuthLayer: AsyncAuthorizeRequest<
            axum::body::Body,
            RequestBody = axum::body::Body,
            ResponseBody = axum::body::Body,
        >
        + Send
        + Sync
        + 'static
        + Clone;

    fn root(&self) -> LandingPage;
    fn add_links(&mut self, links: impl IntoIterator<Item = Link>);

    fn conformance(&self) -> Conformance;
    fn extend_conformance(&self, items: &[&str]);

    fn auth_middleware(&self) -> Self::AuthLayer;
}

#[cfg(feature = "processes")]
pub trait OgcApiProcessesState: OgcApiState {
    fn processors(&self) -> Vec<Box<dyn Processor<User = Self::User>>>;

    fn processor(&self, id: &str) -> Option<Box<dyn Processor<User = Self::User>>>;

    fn jobs(&self) -> &dyn JobHandler<User = Self::User>;
}

#[derive(Clone, Copy)]
pub struct NoAuth;

impl AsyncAuthorizeRequest<axum::body::Body> for NoAuth {
    type RequestBody = axum::body::Body;
    type ResponseBody = axum::body::Body;
    type Future =
        BoxFuture<'static, Result<Request<Self::RequestBody>, Response<Self::ResponseBody>>>;

    fn authorize(&mut self, mut request: Request<Self::RequestBody>) -> Self::Future {
        request.extensions_mut().insert(NoUser);
        Box::pin(async { Ok(request) })
    }
}

impl OgcApiState for AppState {
    type User = NoUser;
    type AuthLayer /*<B: Send + Sync + 'static>*/ = NoAuth;

    fn root(&self) -> LandingPage {
        use crate::util::read_lock;

        read_lock(self.root.as_ref()).to_owned()
    }

    fn add_links(&mut self, links: impl IntoIterator<Item = Link>) {
        use crate::util::write_lock;

        write_lock(&self.root).links.extend(links);
    }

    fn conformance(&self) -> Conformance {
        use crate::util::read_lock;

        read_lock(self.conformance.as_ref()).to_owned()
    }

    fn extend_conformance(&self, items: &[&str]) {
        use crate::util::write_lock;

        write_lock(&self.conformance).extend(items);
    }

    fn auth_middleware(&self) -> Self::AuthLayer {
        NoAuth
    }
}

#[cfg(feature = "processes")]
impl OgcApiProcessesState for AppState {
    fn processors(&self) -> Vec<Box<dyn Processor<User = Self::User>>> {
        use crate::util::read_lock;

        read_lock(self.processors.as_ref())
            .values()
            .cloned()
            .collect()
    }

    fn processor(&self, id: &str) -> Option<Box<dyn Processor<User = Self::User>>> {
        use crate::util::read_lock;

        read_lock(self.processors.as_ref()).get(id).cloned()
    }

    fn jobs(&self) -> &dyn JobHandler<User = Self::User> {
        &*self.drivers.jobs
    }
}
