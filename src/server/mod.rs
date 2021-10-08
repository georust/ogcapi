pub mod routes;

use std::{env, str::FromStr};

pub use routes::{collections, features, processes, styles, tiles};

mod exception;

use tide::{self, http::Mime, utils::After};

use crate::{common::core::MediaType, db::Db};

pub async fn run(url: &str) -> tide::Result<()> {
    tide::log::start();

    let db = Db::connect(&env::var("DATABASE_URL")?).await.unwrap();
    let mut app = tide::with_state(db);

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

    // Collections
    app.at("/collections")
        .get(collections::handle_collections)
        .post(collections::create_collection);
    app.at("/collections/:collection")
        .get(collections::read_collection)
        .put(collections::update_collection)
        .delete(collections::delete_collection);
    //app.at("/collections/:collection/queryables").get(handle_queryables);

    // Features
    app.at("/collections/:collection/items")
        .get(features::handle_items)
        .post(features::create_item);
    app.at("/collections/:collection/items/:id")
        .get(features::read_item)
        .put(features::update_item)
        .delete(features::delete_item);

    // Tiles
    // app.at("tileMatrixSets").get(tiles::get_matrixsets);
    // app.at("tileMatrixSets/:matrix_set").get(tiles::get_matrixset);
    // app.at("collections/:collection/tiles").get(tiles::handle_tiles);
    app.at("collections/:collection/tiles/:matrix_set/:matrix/:row/:col")
        .get(tiles::get_tile);

    // Styles
    app.at("/styles").get(styles::handle_styles);
    // .post(styles::create_style);
    app.at("/styles/:id").get(styles::read_style);
    // .put(styles::update_style)
    // .delete(styles::delete_style);
    // app.at("/styles/:id/metadata").get(styles::read_style_matadata);

    // Processes
    app.at("/processes").get(processes::list_processes);
    app.at("/processes/:id").get(processes::retrieve_process);
    app.at("/processes/:id/execution")
        .post(processes::execution);
    app.at("/jobs/:id")
        .get(processes::job_status)
        .delete(processes::delete_job);
    app.at("/jobs/:id/result").get(processes::job_result);

    app.with(After(exception::exception));

    app.listen(url).await?;
    Ok(())
}

impl Into<Mime> for MediaType {
    fn into(self) -> Mime {
        Mime::from_str(serde_json::to_value(self).unwrap().as_str().unwrap()).unwrap()
    }
}
