use crate::common::link::{ContentType, Link, LinkRelation};
use crate::common::{Conformance, LandingPage};
use crate::features::handles::*;
use crate::routes::{collections, items};
use openapiv3::OpenAPI;
use serde_yaml;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::fs::File;
use tide::http::{url::Position, Url};
use tide::After;

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
                    r#type: Some(ContentType::JSON),
                    title: Some("this document".to_string()),
                    ..Default::default()
                },
                Link {
                    href: "/api".to_string(),
                    rel: LinkRelation::ServiceDesc,
                    r#type: Some(ContentType::OPENAPI),
                    title: Some("the API definition".to_string()),
                    ..Default::default()
                },
                Link {
                    href: "/conformance".to_string(),
                    rel: LinkRelation::Conformance,
                    r#type: Some(ContentType::JSON),
                    title: Some("OGC conformance classes implemented by this API".to_string()),
                    ..Default::default()
                },
                Link {
                    href: "/collections".to_string(),
                    rel: LinkRelation::Data,
                    r#type: Some(ContentType::JSON),
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

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .expect("Create database pool");

        State {
            openapi,
            root,
            conformance,
            pool,
        }
    }
}

pub async fn run(server_url: &str, database_url: &str) -> tide::Result<()> {
    let mut state = State::new(database_url).await;

    state.openapi.servers[0].url = server_url.to_string();

    let server_url = Url::parse(&server_url)?;

    tide::log::start();

    let mut app = tide::with_state(state);

    app.at("/").get(handle_root);
    app.at("/api").get(handle_api);
    app.at("/conformance").get(handle_conformance);
    app.at("/favicon.ico").get(handle_favicon);
    app.at("/redoc").get(show_redoc);

    app.at("/collections")
        .get(collections::handle_collections)
        .post(collections::handle_collection);
    app.at("/collections/:collection")
        .get(collections::handle_collection)
        .put(collections::handle_collection)
        .delete(collections::handle_collection);

    app.at("/collections/:collection/items")
        .get(items::handle_items)
        .post(items::handle_item);
    app.at("/collections/:collection/items/:id")
        .get(items::handle_item)
        .put(items::handle_item)
        .delete(items::handle_item);

    app.middleware(After(exception));

    app.listen(&server_url[Position::BeforeHost..Position::AfterPort])
        .await?;
    Ok(())
}
