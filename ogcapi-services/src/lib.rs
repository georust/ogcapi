mod config;
mod error;
mod extractors;
#[cfg(feature = "processes")]
mod processor;
mod routes;

pub use config::{parse_config, Config};
pub use error::Error;
#[cfg(feature = "processes")]
pub use processor::Processor;

use std::sync::{Arc, RwLock};

use axum::{extract::Extension, routing::get, Router};
use openapiv3::OpenAPI;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

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

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub static OPENAPI: &[u8; 29696] = include_bytes!("../openapi.yaml");

// #[derive(Clone)]
pub struct State {
    pub drivers: Drivers,
    pub root: RwLock<LandingPage>,
    pub conformance: RwLock<Conformance>,
    pub openapi: OpenAPI,
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

pub async fn app(db: Db, api: &[u8]) -> Router {
    // state
    let openapi: OpenAPI = serde_yaml::from_slice(api).unwrap();

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

    let state = State {
        drivers,
        root,
        conformance,
        openapi,
    };

    // routes
    let router = Router::new()
        .route("/", get(routes::root))
        .route("/api", get(routes::api::api))
        .route("/redoc", get(routes::api::redoc))
        .route("/swagger", get(routes::api::swagger))
        .route("/conformance", get(routes::conformance));

    let router = router.merge(routes::collections::router(&state));

    #[cfg(feature = "features")]
    let router = router.merge(routes::features::router(&state));

    #[cfg(feature = "edr")]
    let router = router.merge(routes::edr::router(&state));

    #[cfg(feature = "styles")]
    let router = router.merge(routes::styles::router(&state));

    #[cfg(feature = "tiles")]
    let router = router.merge(routes::tiles::router(&state));

    #[cfg(feature = "processes")]
    let router = router.merge(routes::processes::router(
        &state,
        vec![
            Box::new(processor::Greeter),
            // Box::new(processor::AssetLoader),
        ],
    ));

    // middleware stack
    router.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new())
            .layer(CorsLayer::permissive())
            .layer(Extension(Arc::new(state))),
    )
}

/// Handle shutdown signals
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::debug!("signal received, starting graceful shutdown");
}
