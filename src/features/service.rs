use super::routes;
use crate::common::{self, Conformance};
use openapiv3::OpenAPI;
use routes::{collections, items};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::fs::File;
use tide::{
    http::{url::Position, Url},
    utils::After,
    Body, Request, Response,
};

static API: &str = "api/ogcapi-features-1.yaml";

#[derive(Clone)]
pub struct Features {
    pub conformance: Conformance,
    pub api: OpenAPI,
    pub pool: PgPool,
}

impl Features {
    pub async fn new(db: &str) -> Features {
        Features {
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
                .connect(db)
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
        app.at("/favicon.ico").get(|_: Request<Features>| async {
            let mut res = Response::new(200);
            res.set_body(Body::from_file("favicon.ico").await?);
            Ok(res)
        });

        // redoc
        app.at("/redoc").get(routes::redoc);

        // collections
        app.at("/collections")
            .get(collections::handle_collections)
            .post(collections::create_collection);
        app.at("/collections/:collection")
            .get(collections::read_collection)
            .put(collections::update_collection)
            .delete(collections::delete_collection);

        // items
        app.at("/collections/:collection/items")
            .get(items::handle_items)
            .post(items::create_item);
        app.at("/collections/:collection/items/:id")
            .get(items::read_item)
            .put(items::update_item)
            .delete(items::delete_item);

        app.with(After(common::exception));

        app.listen(&url[Position::BeforeHost..Position::AfterPort])
            .await?;
        Ok(())
    }
}
