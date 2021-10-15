pub mod routes;

use std::{collections::HashMap, str::FromStr, sync::Arc};

use async_std::sync::RwLock;
use tide::{self, http::Mime, utils::After, Body, Response, Result};
use url::Url;

use crate::common::collections::Collection;
use crate::common::core::{Exception, MediaType};
use crate::db::Db;

#[derive(Clone)]
pub(crate) struct State {
    db: Db,
    collections: Arc<RwLock<HashMap<String, Collection>>>,
}

pub async fn run(host: &str, port: &str, database_url: &Url) -> Result<()> {
    tide::log::with_level(tide::log::LevelFilter::from_str(
        dotenv::var("RUST_LOG")?.as_str(),
    )?);

    let state = State {
        db: Db::connect(database_url.as_str()).await.unwrap(),
        collections: Default::default(),
    };

    let mut app = tide::with_state(state.clone());

    app.at("/").get(routes::root);
    app.at("/api").get(routes::api);
    app.at("/redoc").get(routes::redoc);
    app.at("/conformance").get(routes::conformance);
    app.at("/favicon.ico").serve_file("favicon.ico")?;

    routes::collections::register(&mut app);
    routes::features::register(&mut app);
    routes::edr::register(&mut app);
    routes::tiles::register(&mut app);
    routes::styles::register(&mut app);
    routes::processes::register(&mut app);

    app.with(After(|mut res: Response| async move {
        if let Some(err) = res.error() {
            let exception = Exception {
                r#type: format!(
                    "https://httpwg.org/specs/rfc7231.html#status.{}",
                    res.status().to_string()
                ),
                status: Some(res.status() as isize),
                // NOTE: You may want to avoid sending error messages in a production server.
                detail: Some(err.to_string()),
                ..Default::default()
            };
            res.set_body(Body::from_json(&exception)?);
            res.set_content_type(MediaType::ProblemJSON);
        }
        Ok(res)
    }));

    app.listen(&format!("{}:{}", host, port)).await?;

    Ok(())
}

impl Into<Mime> for MediaType {
    fn into(self) -> Mime {
        Mime::from_str(serde_json::to_value(self).unwrap().as_str().unwrap()).unwrap()
    }
}
