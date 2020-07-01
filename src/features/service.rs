use crate::features::handles::*;
use openapiv3::OpenAPI;
use serde_yaml;
use sqlx::postgres::PgPool;
use std::fs::File;
use tide::http::{url::Position, Url};
use tide::After;

use crate::common::{Conformance, ContentType, LandingPage, Link, LinkRelation};

pub struct State {
    pub openapi: OpenAPI,
    pub root: LandingPage,
    pub conformance: Conformance,
    pub pool: PgPool,
}

impl State {
    async fn new(database_url: &str) -> State {
        let api = "api/ogcapi-features-1.yaml";
        let api = File::open(api).expect("Open api file");
        let openapi: OpenAPI = serde_yaml::from_reader(api).expect("Deserialize api document");

        let root = LandingPage {
            title: Some(openapi.info.title.clone()),
            description: openapi.info.description.clone(),
            links: vec![
                Link {
                    href: "/".to_string(),
                    r#type: Some(ContentType::Json),
                    title: Some("this document".to_string()),
                    ..Default::default()
                },
                Link {
                    href: "/api".to_string(),
                    rel: LinkRelation::ServiceDesc,
                    r#type: Some(ContentType::OpenAPI),
                    title: Some("the API definition".to_string()),
                    ..Default::default()
                },
                Link {
                    href: "/conformance".to_string(),
                    rel: LinkRelation::Conformance,
                    r#type: Some(ContentType::Json),
                    title: Some("OGC conformance classes implemented by this API".to_string()),
                    ..Default::default()
                },
                Link {
                    href: "/collections".to_string(),
                    rel: LinkRelation::Data,
                    r#type: Some(ContentType::Json),
                    title: Some("Metadata about the resource collections".to_string()),
                    ..Default::default()
                },
            ],
        };

        let conformance = Conformance {
            conforms_to: vec![
                "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core".to_string(),
                "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30".to_string(),
                "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson".to_string(),
            ],
        };

        let pool = PgPool::new(database_url).await.expect("Create pg pool");

        State {
            openapi,
            root,
            conformance,
            pool,
        }
    }
}

// pub struct Service {
//     app: tide::Server<State>,
//     url: Url,
// }

pub async fn run(server_url: &str, database_url: &str) -> tide::Result<()> {
    let mut state = State::new(database_url).await;

    state.openapi.servers[0].url = server_url.to_string();

    let server_url = Url::parse(&server_url)?;

    tide::log::start();

    let mut app = tide::with_state(state);

    app.at("/").get(handle_root);
    app.at("/api").get(handle_api);
    app.at("/conformance").get(handle_conformance);

    app.at("/collections").get(handle_collections);
    app.at("/collections/:collection")
        .get(handle_collection)
        .post(handle_collection)
        .put(handle_collection)
        .delete(handle_collection);

    app.at("/collections/:collection/items").get(handle_items);
    app.at("/collections/:collection/items/:id")
        .get(handle_item);
    // .post(handle_item)
    // .put(handle_item)
    // .delete(handle_item);

    app.at("/favicon.ico").get(handle_favicon);

    app.middleware(After(exception));

    app.listen(&server_url[Position::BeforeHost..Position::AfterPort])
        .await?;
    Ok(())
}
