pub mod collections;
#[cfg(feature = "edr")]
pub mod edr;
pub mod features;
pub mod processes;
pub mod styles;
pub mod tiles;

use std::sync::Arc;

use axum::{
    extract::Extension, headers::HeaderMap, http::header::CONTENT_TYPE, response::Html, Json,
};
use openapiv3::OpenAPI;

use ogcapi_types::common::{media_type::OPEN_API_JSON, Conformance, LandingPage};

use crate::{extractors::RemoteUrl, Result, State};

pub(crate) async fn root(Extension(state): Extension<Arc<State>>) -> Result<Json<LandingPage>> {
    Ok(Json(state.root.read().unwrap().clone()))
}

pub(crate) async fn api(Extension(state): Extension<Arc<State>>) -> (HeaderMap, Json<OpenAPI>) {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, OPEN_API_JSON.parse().unwrap());

    (headers, Json(state.openapi.to_owned()))
}

pub(crate) async fn redoc(RemoteUrl(url): RemoteUrl) -> Result<Html<String>> {
    let api = url.join("../api")?;

    Ok(Html(format!(
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
        &api
    )))
}

pub(crate) async fn conformance(Extension(state): Extension<Arc<State>>) -> Json<Conformance> {
    Json(state.conformance.read().unwrap().to_owned())
}
