mod config;
mod error;
mod extractors;
#[cfg(feature = "processes")]
mod processor;
mod routes;

pub use config::Config;
pub use error::Error;
#[cfg(feature = "processes")]
pub use processor::Processor;

use std::sync::{Arc, RwLock};

use axum::{extract::Extension, routing::get, Router};
use openapiv3::OpenAPI;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use ogcapi_drivers::{
    postgres::Db, CollectionTransactions, EdrQuerier, FeatureTransactions, JobHandler,
    StyleTransactions, TileTransactions,
};
use ogcapi_types::common::{
    link_rel::{CONFORMANCE, SELF, SERVICE_DESC},
    media_type::{JSON, OPEN_API_JSON},
    Conformance, LandingPage, Link,
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

static OPENAPI: &[u8; 29758] = include_bytes!("../openapi.yaml");

// #[derive(Clone)]
struct State {
    drivers: Drivers,
    // collections: RwLock<HashMap<String, Collection>>,
    root: RwLock<LandingPage>,
    conformance: RwLock<Conformance>,
    openapi: OpenAPI,
    remote: String,
}

struct Drivers {
    collections: Box<dyn CollectionTransactions>,
    features: Box<dyn FeatureTransactions>,
    edr: Box<dyn EdrQuerier>,
    jobs: Box<dyn JobHandler>,
    styles: Box<dyn StyleTransactions>,
    tiles: Box<dyn TileTransactions>,
}

pub async fn app(db: Db) -> Router {
    // state
    let openapi: OpenAPI = serde_yaml::from_slice(OPENAPI).unwrap();
    let remote = openapi.servers[0].url.to_owned();

    let root = RwLock::new(LandingPage {
        title: Some(openapi.info.title.to_owned()),
        description: openapi.info.description.to_owned(),
        links: vec![
            Link::new(&remote, SELF).title("This document").mime(JSON),
            Link::new(format!("{}/api", &remote), SERVICE_DESC)
                .title("The Open API definition")
                .mime(OPEN_API_JSON),
            Link::new(format!("{}/conformance", &remote), CONFORMANCE)
                .title("OGC conformance classes implemented by this API")
                .mime(JSON),
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
        features: Box::new(db.clone()),
        edr: Box::new(db.clone()),
        jobs: Box::new(db.clone()),
        styles: Box::new(db.clone()),
        tiles: Box::new(db),
    };

    let state = State {
        drivers,
        root,
        conformance,
        openapi,
        remote,
    };

    // routes
    let router = Router::new()
        .route("/", get(routes::root))
        .route("/api", get(routes::api))
        .route("/redoc", get(routes::redoc))
        .route("/conformance", get(routes::conformance))
        .merge(routes::collections::router(&state))
        .merge(routes::features::router(&state))
        .merge(routes::tiles::router(&state))
        .merge(routes::styles::router(&state));

    #[cfg(feature = "processes")]
    let processors = vec![processor::Greeter];
    let router = router.merge(routes::processes::router(&state, processors));

    #[cfg(feature = "edr")]
    let router = router.merge(routes::edr::router(&state));

    router.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
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
