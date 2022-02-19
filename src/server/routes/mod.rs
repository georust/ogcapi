pub mod collections;
#[cfg(feature = "edr")]
pub mod edr;
pub mod features;
#[cfg(feature = "processes")]
pub mod processes;
pub mod styles;
pub mod tiles;

use std::str::FromStr;

use http::Uri;
use tide::{
    http::{url::Position, Mime},
    Body, Request, Response,
};
use url::Url;

use crate::common::core::MediaType;
use crate::server::State;

pub(crate) async fn root(req: Request<State>) -> tide::Result {
    let url = req
        .remote()
        .and_then(|s| Url::from_str(s).ok())
        .unwrap_or_else(|| req.url().to_owned());

    let mut landing_page = req.state().root.read().await.clone();

    for link in landing_page.links.iter_mut() {
        let uri = Uri::builder()
            .scheme(url.scheme())
            .authority({
                let mut authority = url.host_str().unwrap().to_owned();
                if let Some(port) = url.port() {
                    authority.push_str(&format!(":{}", port));
                }
                authority
            })
            .path_and_query(
                link.href
                    .parse::<Uri>()
                    .unwrap()
                    .path_and_query()
                    .unwrap()
                    .as_str(),
            )
            .build()?;
        link.href = uri.to_string();
    }

    Ok(Response::builder(200)
        .body(Body::from_json(&landing_page)?)
        .build())
}

pub(crate) async fn api(req: Request<State>) -> tide::Result {
    Ok(Response::builder(200)
        .body(Body::from_json(&req.state().openapi)?)
        .content_type(MediaType::OpenAPIJson)
        .build())
}

pub(crate) async fn redoc(req: Request<State>) -> tide::Result {
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

pub(crate) async fn conformance(req: Request<State>) -> tide::Result {
    let conformance = req.state().conformance.read().await;
    Ok(Response::builder(200)
        .body(Body::from_json(&conformance.to_owned())?)
        .build())
}
