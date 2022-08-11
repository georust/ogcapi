use axum::{extract::State, headers::HeaderMap, http::header::CONTENT_TYPE, response::Html, Json};
use openapiv3::OpenAPI;

use ogcapi_types::common::media_type::OPEN_API_JSON;

use crate::{AppState, Result};

pub(crate) async fn api(State(state): State<AppState>) -> (HeaderMap, Json<OpenAPI>) {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, OPEN_API_JSON.parse().unwrap());

    (headers, Json(state.openapi.0))
}

pub(crate) async fn redoc() -> Result<Html<String>> {
    Ok(Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>ReDoc</title>
            <!-- needed for adaptive design -->
            <meta charset="utf-8"/>
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <link href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700" rel="stylesheet">
            <style>
                body {
                    margin: 0;
                    padding: 0;
                }
            </style>
        </head>
        <body>
            <redoc spec-url="api"></redoc>
            <script src="https://cdn.jsdelivr.net/npm/redoc@next/bundles/redoc.standalone.js"></script>
        </body>
        </html>
        "#.to_string(),
    ))
}

pub(crate) async fn swagger() -> Result<Html<String>> {
    Ok(Html(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="utf-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1" />
            <meta name="description" content="SwaggerIU" />
            <title>SwaggerUI</title>
            <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@4.11.1/swagger-ui.css" />
        </head>
        <body>
            <div id="swagger-ui"></div>
            <script src="https://unpkg.com/swagger-ui-dist@4.11.1/swagger-ui-bundle.js" crossorigin></script>
            <script>
            window.onload = () => {
                window.ui = SwaggerUIBundle({
                url: 'api',
                dom_id: '#swagger-ui',
                });
            };
            </script>
        </body>
        </html>
        "#.to_string(),
    ))
}
