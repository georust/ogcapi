use super::Service;
use crate::common::{Conformance, ContentType, LandingPage, Link, LinkRelation};
use openapiv3::OpenAPI;
use serde_json::json;
use std::env;
use std::fs::File;
use tide::{http::url::Position, Body, Request, Response, Result};

pub async fn root(req: Request<Service>) -> Result {
    let url = req.url();

    let api_definition = env::var("API_DEFINITION")?;
    let rdr = File::open(&api_definition)?;
    let openapi: OpenAPI = serde_yaml::from_reader(rdr)?;

    let mut landing_page = LandingPage {
        title: Some(openapi.info.title),
        description: openapi.info.description,
        links: vec![
            Link {
                href: "/".to_string(),
                r#type: Some(ContentType::JSON),
                title: Some("this document".to_string()),
                ..Default::default()
            },
            Link {
                href: "/api".to_string(),
                rel: LinkRelation::ServiceDesc,
                r#type: Some(ContentType::OPENAPI),
                title: Some("the API definition".to_string()),
                ..Default::default()
            },
            Link {
                href: "/conformance".to_string(),
                rel: LinkRelation::Conformance,
                r#type: Some(ContentType::JSON),
                title: Some("OGC conformance classes implemented by this API".to_string()),
                ..Default::default()
            },
            Link {
                href: "/collections".to_string(),
                rel: LinkRelation::Data,
                r#type: Some(ContentType::JSON),
                title: Some("Metadata about the resource collections".to_string()),
                ..Default::default()
            },
        ],
        attribution: None,
    };

    for link in landing_page.links.iter_mut() {
        link.href = format!("{}{}", url, link.href.trim_matches('/'));
    }

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&landing_page)?);
    Ok(res)
}

pub async fn api(_req: Request<Service>) -> Result {
    let api_definition = env::var("API_DEFINITION")?;
    let rdr = File::open(&api_definition)?;
    let openapi: OpenAPI = serde_yaml::from_reader(rdr)?;

    let mut res = Response::new(200);
    // res.set_content_type(ContentType::OPENAPI);
    res.set_body(Body::from_json(&openapi)?);
    Ok(res)
}

pub async fn redoc(req: Request<Service>) -> Result {
    let api_url = req.url()[..Position::AfterPath].replace("redoc", "api");

    let mut res = Response::new(200);
    res.set_content_type(tide::http::mime::HTML);
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
        </html>
        "#,
        api_url
    ));
    Ok(res)
}

pub async fn conformance(_req: Request<Service>) -> Result {
    let conformance: Conformance = serde_json::from_value(json!({
        "conformsTo": [
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core",
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30",
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson",
            "http://www.opengis.net/spec/ogcapi-features-2/1.0/conf/crs",
        ]
    }))?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&conformance)?);
    Ok(res)
}
