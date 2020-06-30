use crate::features::handles::*;
use openapiv3::OpenAPI;
use serde::{Deserialize, Serialize};
use serde_yaml;
use sqlx::postgres::PgPool;
use std::fs::File;
use tide::http::{url::Position, Url};
use tide::After;

use crate::common::Link;
use crate::features::schema::Conformance;

pub struct State {
    pub api: OpenAPI,
    pub config: Config,
    pub pool: PgPool,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub conformance: Conformance,
    pub root_links: Vec<Link>,
}

impl State {
    async fn new(api: &str, db_url: &str) -> State {
        let config = File::open("Config.json").expect("Open config");
        let config: Config = serde_json::from_reader(config).expect("Deserialize config");
        let api = File::open(api).expect("Open api file");
        let api: OpenAPI = serde_yaml::from_reader(api).expect("Deserialize api document");
        let pool = PgPool::new(db_url).await.expect("Create pg pool");
        State { api, config, pool }
    }
}

pub struct Service {
    app: tide::Server<State>,
    url: Url,
}

impl Service {
    pub async fn new(db_url: &str) -> Service {
        let state = State::new("api/ogcapi-features-1.yaml", db_url).await;

        let url = Url::parse(&state.api.servers[0].url).expect("Parse url from string");

        let mut app = tide::with_state(state);

        app.middleware(After(exception));

        app.at("/").get(handle_root);
        app.at("/api").get(handle_api);
        app.at("/conformance").get(handle_conformance);
        app.at("/collections").get(handle_collections);
        app.at("/collections/:collection").get(handle_collection);
        app.at("/collections/:collection/items").get(handle_items);
        app.at("/collections/:collection/items/:id")
            .get(handle_item);

        Service { app, url }
    }

    pub async fn run(self) -> tide::Result<()> {
        self.app
            .listen(&self.url[Position::BeforeHost..Position::AfterPort])
            .await?;
        Ok(())
    }
}
