mod error;
mod extractors;
mod routes;

pub use error::Error;

use std::sync::{Arc, RwLock};

use axum::{extract::Extension, routing::get, Router};
use openapiv3::OpenAPI;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use ogcapi_drivers::postgres::Db;
use ogcapi_types::common::{
    link_rel::{CONFORMANCE, SELF, SERVICE_DESC},
    media_type::{JSON, OPEN_API_JSON},
    Conformance, LandingPage, Link,
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

static OPENAPI: &[u8; 29758] = include_bytes!("../openapi.yaml");

#[derive(Clone)]
struct State {
    db: Db,
    // collections: Arc<RwLock<HashMap<String, Collection>>>,
    root: Arc<RwLock<LandingPage>>,
    conformance: Arc<RwLock<Conformance>>,
    openapi: Arc<OpenAPI>,
    remote: Arc<String>,
}

pub async fn server(db: Db) -> Router {
    // state
    let openapi: OpenAPI = serde_yaml::from_slice(OPENAPI).unwrap();
    let remote = openapi.servers[0].url.to_owned();

    let root = Arc::new(RwLock::new(LandingPage {
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
    }));

    let conformance = Arc::new(RwLock::new(Conformance {
        conforms_to: vec![
            "http://www.opengis.net/spec/ogcapi-common-1/1.0/req/core".to_string(),
            "http://www.opengis.net/spec/ogcapi-common-2/1.0/req/collections".to_string(),
            "http://www.opengis.net/spec/ogcapi_common-2/1.0/req/json".to_string(),
        ],
    }));

    let state = State {
        db,
        root,
        conformance,
        openapi: Arc::new(openapi),
        remote: Arc::new(remote),
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
    let router = router.merge(routes::processes::router(&state));

    #[cfg(feature = "edr")]
    let router = router.merge(routes::edr::router(&state));

    router.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive())
            .layer(Extension(state)),
    )
}
