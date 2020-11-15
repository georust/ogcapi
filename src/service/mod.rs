mod routes;

use crate::common::{self, Conformance};
use crate::{collections, features};
use openapiv3::OpenAPI;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use std::fs::File;
use tide::{
    http::{url::Position, Url},
    utils::After,
    Body, Request, Response,
};

static API: &str = "api/ogcapi-features_sprint.yaml";

#[derive(Clone)]
pub struct Service {
    pub conformance: Conformance,
    pub api: OpenAPI,
    pub pool: PgPool,
}

impl Service {
    pub async fn new() -> Service {
        Service {
            api: serde_yaml::from_reader(File::open(API).expect("Open api file"))
                .expect("Deserialize api document"),
            conformance: Conformance {
                conforms_to: vec![
                    "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core".to_string(),
                    "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30".to_string(),
                    "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson".to_string(),
                ],
            },
            pool: PgPoolOptions::new()
                .max_connections(5)
                .connect(&env::var("DATABASE_URL").unwrap())
                .await
                .expect("Create database pool"),
        }
    }
    pub async fn run(mut self, url: &str) -> tide::Result<()> {
        let url = Url::parse(&url)?;
        self.api.servers[0].url = url.to_string();

        tide::log::start();
        let mut app = tide::with_state(self);

        // core
        app.at("/").get(routes::root);
        app.at("/api").get(routes::api);
        app.at("/conformance").get(routes::conformance);

        // favicon
        app.at("/favicon.ico").get(|_: Request<Service>| async {
            let mut res = Response::new(200);
            res.set_body(Body::from_file("favicon.ico").await?);
            Ok(res)
        });

        // redoc
        app.at("/redoc").get(routes::redoc);

        // queryables
        //app.at("/queryables").get(handle_queryables);

        // collections
        app.at("/collections")
            .get(collections::handle_collections)
            .post(collections::create_collection);
        app.at("/collections/:collection")
            .get(collections::read_collection)
            .put(collections::update_collection)
            .delete(collections::delete_collection);
        //app.at("/collections/:collection/queryables").get(handle_queryables);

        // items
        app.at("/collections/:collection/items")
            .get(features::handle_items)
            .post(features::create_item);
        app.at("/collections/:collection/items/:id")
            .get(features::read_item)
            .put(features::update_item)
            .delete(features::delete_item);

        app.with(After(common::exception));

        app.listen(&url[Position::BeforeHost..Position::AfterPort])
            .await?;
        Ok(())
    }
}
