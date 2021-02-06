mod routes;

use crate::{collections, common, features, tiles};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use tide::{self, utils::After};

#[derive(Clone)]
pub struct Service {
    pub pool: PgPool,
}

impl Service {
    pub async fn new() -> Self {
        Service {
            pool: PgPoolOptions::new()
                .max_connections(5)
                .connect(&env::var("DATABASE_URL").expect("Read database url"))
                .await
                .expect("Create db connection pool"),
        }
    }

    pub async fn run(self, url: &str) -> tide::Result<()> {
        tide::log::start();
        let mut app = tide::with_state(self);

        // core
        app.at("/").get(routes::root);
        app.at("/api").get(routes::api);
        app.at("/conformance").get(routes::conformance);

        // favicon
        app.at("/favicon.ico").serve_file("favicon.ico")?;

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

        app.listen(url).await?;
        Ok(())
    }
}
