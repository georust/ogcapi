pub mod collections;
pub mod items;

use crate::common::{ContentType, LandingPage, Link, LinkRelation};
use crate::Features;
use tide::{Body, Request, Response, Result};

pub async fn root(req: Request<Features>) -> Result {
    let url = req.url();

    let openapi = &req.state().api;

    let mut landing_page = LandingPage {
        title: Some(openapi.info.title.clone()),
        description: openapi.info.description.clone(),
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
    };

    for link in landing_page.links.iter_mut() {
        link.href = format!("{}{}", url, link.href.trim_matches('/'));
    }

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&landing_page)?);
    Ok(res)
}

pub async fn api(req: Request<Features>) -> Result {
    let mut res = Response::new(200);
    res.set_content_type(ContentType::OPENAPI);
    res.set_body(Body::from_json(&req.state().api)?);
    Ok(res)
}

pub async fn redoc(req: Request<Features>) -> Result {
    let mut url = req.url().to_owned();

    url.set_query(None);
    let api_url = url.to_string().replace("redoc", "api");

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

pub async fn conformance(req: Request<Features>) -> Result {
    let mut res = Response::new(200);
    res.set_body(Body::from_json(&req.state().conformance)?);
    Ok(res)
}
