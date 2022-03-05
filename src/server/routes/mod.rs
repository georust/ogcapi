pub mod collections;
#[cfg(feature = "edr")]
pub mod edr;
pub mod features;
pub mod processes;
pub mod styles;
pub mod tiles;

use std::sync::Arc;

use axum::{extract::Extension, http::Uri, response::Headers, response::Html, Json};
use openapiv3::OpenAPI;
use url::Url;

use crate::common::core::{Conformance, LandingPage, MediaType};
use crate::server::{Error, Result, State};

pub(crate) async fn root(Extension(state): Extension<State>) -> Result<Json<LandingPage>> {
    // TODO: create custom extractor
    let url = Url::parse(&format!("http://localhost:8484{}", "")).unwrap();

    let mut landing_page = state.root.read().unwrap().clone();

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
            .build()
            .map_err(|e| Error::Anyhow(e.into()))?;
        link.href = uri.to_string();
    }

    Ok(Json(landing_page))
}

pub(crate) async fn api(
    Extension(state): Extension<State>,
) -> (Headers<Vec<(&'static str, String)>>, Json<Arc<OpenAPI>>) {
    let headers = Headers(vec![("Content-Type", MediaType::OpenAPIJson.to_string())]);

    (headers, Json(state.openapi))
}

pub(crate) async fn redoc() -> Html<String> {
    // TODO: create custom extractor
    let api_url = Url::parse(&format!("http://localhost:8484/{}", "api")).unwrap();

    Html(format!(
        r#"<!DOCTYPE html>
        <html>
        <head>
            <title>ReDoc</title>
            <!-- needed for adaptive design -->
            <meta charset="utf-8"/>
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <link href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700" rel="stylesheet">
            <style>
                body {{
                    margin: 0;
                    padding: 0;
                }}
            </style>
        </head>
        <body>
            <redoc spec-url="{}"></redoc>
            <script src="https://cdn.jsdelivr.net/npm/redoc@next/bundles/redoc.standalone.js"></script>
        </body>
        </html>"#,
        api_url
    ))
}

pub(crate) async fn conformance(Extension(state): Extension<State>) -> Json<Conformance> {
    Json(state.conformance.read().unwrap().to_owned())
}
