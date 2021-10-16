pub mod collections;
pub mod edr;
pub mod features;
pub mod processes;
pub mod styles;
pub mod tiles;

use openapiv3::OpenAPI;
use tide::{
    http::{url::Position, Mime},
    Body, Request, Response, Result,
};

use crate::common::core::{LandingPage, MediaType};
use crate::server::State;

static OPENAPI: &[u8; 29680] = include_bytes!("../../../openapi.yaml");

pub(crate) async fn root(req: Request<State>) -> Result {
    let url = req.url();

    let openapi: OpenAPI = serde_yaml::from_slice(OPENAPI)?;

    let links = req.state().db.root().await?;

    let mut landing_page = LandingPage {
        title: Some(openapi.info.title),
        description: openapi.info.description,
        links,
        ..Default::default()
    };

    for link in landing_page.links.iter_mut() {
        link.url.set_scheme(url.scheme()).unwrap();
        link.url.set_host(url.host_str()).unwrap();
        link.url.set_port(url.port()).unwrap();
    }

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&landing_page)?);
    Ok(res)
}

pub(crate) async fn api(_req: Request<State>) -> Result {
    let openapi: OpenAPI = serde_yaml::from_slice(OPENAPI)?;

    let mut res = Response::new(200);
    res.set_content_type(MediaType::OpenAPI);
    res.set_body(Body::from_json(&openapi)?);
    Ok(res)
}

pub(crate) async fn redoc(req: Request<State>) -> Result {
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

pub(crate) async fn conformance(req: Request<State>) -> Result {
    let conformance = req.state().db.conformance().await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&conformance)?);
    Ok(res)
}
