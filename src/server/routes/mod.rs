pub mod collections;
pub mod features;
pub mod processes;
pub mod styles;
pub mod tiles;

use std::fs::File;

use openapiv3::OpenAPI;
use tide::{
    http::{url::Position, Mime},
    Body, Request, Response, Result,
};

use crate::{common::core::LandingPage, db::Db};

static OPENAPI: &'static str = "openapi.yaml";

pub async fn root(req: Request<Db>) -> Result {
    let url = req.url();

    let rdr = File::open(OPENAPI)?;
    let openapi: OpenAPI = serde_yaml::from_reader(rdr)?;

    let links = req.state().root().await?;

    let mut landing_page = LandingPage {
        title: Some(openapi.info.title),
        description: openapi.info.description,
        links,
        attribution: None,
    };

    for link in landing_page.links.iter_mut() {
        link.href = format!("{}{}", url, link.href.trim_matches('/'));
    }

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&landing_page)?);
    Ok(res)
}

pub async fn api(_req: Request<Db>) -> Result {
    let rdr = File::open(OPENAPI)?;
    let openapi: OpenAPI = serde_yaml::from_reader(rdr)?;

    let mut res = Response::new(200);
    // res.set_content_type(ContentType::OpenAPI);
    res.set_body(Body::from_json(&openapi)?);
    Ok(res)
}

pub async fn redoc(req: Request<Db>) -> Result {
    let api_url = req.url()[..Position::AfterPath].replace("redoc", "api");

    let mut res = Response::new(200);
    res.set_content_type(Mime::from("text/html"));
    res.set_body(format!(
        r#"<!DOCTYPE html>
        <html>
        <head>
            <title>ReDoc</title>
            <!-- needed for adaptive design -->
            <meta charset="utf-8"/>
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <link href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700" rel="stylesheet">

            <!--
            ReDoc doesn't change outer page styles
            -->
            <style>
            body {{
                margin: 0;
                padding: 0;
            }}
            </style>
        </head>
        <body>
            <redoc spec-url="{}"></redoc>
            <script src="https://cdn.jsdelivr.net/npm/redoc@next/bundles/redoc.standalone.js"> </script>
        </body>
        </html>"#,
        api_url
    ));
    Ok(res)
}

pub async fn conformance(req: Request<Db>) -> Result {
    let conformance = req.state().conformance().await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&conformance)?);
    Ok(res)
}
