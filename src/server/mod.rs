pub mod routes;

use std::{str::FromStr, sync::Arc};

use async_std::sync::RwLock;
use openapiv3::OpenAPI;
use tide::Server;
use tide::{self, http::Mime, utils::After, Body, Response};
use url::Url;

use crate::common::core::{Conformance, Exception, LandingPage, Link, LinkRel, MediaType};
use crate::db::Db;

static OPENAPI: &[u8; 29680] = include_bytes!("../../openapi.yaml");

#[derive(Clone)]
pub struct State {
    db: Db,
    // collections: Arc<RwLock<HashMap<String, Collection>>>,
    root: Arc<RwLock<LandingPage>>,
    conformance: Arc<RwLock<Conformance>>,
    openapi: OpenAPI,
}

pub async fn server(database_url: &Url) -> Server<State> {
    // log
    tide::log::with_level(
        tide::log::LevelFilter::from_str(
            dotenv::var("RUST_LOG").expect("Read RUST_LOG env").as_str(),
        )
        .expect("Setup rust log level"),
    );

    // state
    let db = Db::connect(database_url.as_str()).await.unwrap();

    let openapi: OpenAPI = serde_yaml::from_slice(OPENAPI).unwrap();

    let root = Arc::new(RwLock::new(LandingPage {
        title: Some(openapi.info.title.clone()),
        description: openapi.info.description.clone(),
        links: vec![
            Link::new("http://ogcapi.rs/")
                .title("This document".to_string())
                .mime(MediaType::JSON),
            Link::new("http://ogcapi.rs/api")
                .title("The Open API definition".to_string())
                .relation(LinkRel::ServiceDesc)
                .mime(MediaType::OpenAPIJson),
            Link::new("http://ogcapi.rs/conformance")
                .title("OGC conformance classes implemented by this API".to_string())
                .relation(LinkRel::Conformance)
                .mime(MediaType::JSON),
            Link::new("http://ogcapi.rs/collections")
                .title("Metadata about the resource collections".to_string())
                .relation(LinkRel::Data)
                .mime(MediaType::JSON),
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
        openapi,
    };

    // routes
    let mut app = tide::with_state(state.clone());

    app.at("/").get(routes::root);
    app.at("/api").get(routes::api);
    app.at("/redoc").get(routes::redoc);
    app.at("/conformance").get(routes::conformance);

    app.at("/favicon.ico").get(|_| async move {
        Ok(Response::from(Body::from_bytes(
            include_bytes!("../../favicon.ico").to_vec(),
        )))
    });

    routes::collections::register(&mut app).await;
    routes::features::register(&mut app).await;
    #[cfg(feature = "edr")]
    routes::edr::register(&mut app).await;
    routes::tiles::register(&mut app);
    routes::styles::register(&mut app);
    #[cfg(feature = "processes")]
    routes::processes::register(&mut app).await;

    // errors
    app.with(After(|mut res: Response| async move {
        if let Some(err) = res.error() {
            let exception = Exception {
                r#type: format!(
                    "https://httpwg.org/specs/rfc7231.html#status.{}",
                    res.status()
                ),
                status: Some(res.status() as isize),
                // NOTE: You may want to avoid sending error messages in a production server.
                detail: Some(err.to_string()),
                ..Default::default()
            };
            res.set_body(Body::from_json(&exception).expect("Serialize exception"));
            res.set_content_type(MediaType::ProblemJSON);
        }
        Ok(res)
    }));

    app
}

// impl Into<Mime> for MediaType {
//     fn into(self) -> Mime {
//         Mime::from_str(serde_json::to_value(self).unwrap().as_str().unwrap()).unwrap()
//     }
// }

impl From<MediaType> for Mime {
    fn from(m: MediaType) -> Self {
        Mime::from_str(serde_json::to_value(m).unwrap().as_str().unwrap()).unwrap()
    }
}
