use async_std::task;
use openapiv3::{OpenAPI, Server};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;
use tide::prelude::*;
use tide::{Body, Request, Response};

use oapi::schema::*;
#[derive(Deserialize, Serialize)]
struct Cat {
    name: String,
}

async fn handle_root(req: Request<()>) -> tide::Result {
    let root = req.url();
    let body = serde_json::to_string_pretty(&LandingPage {
        title: None,
        description: None,
        links: vec![
            Link {
                href: format!("{}/", root),
                rel: Some(String::from("self")),
                r#type: Some(String::from("application/json")),
                title: Some(String::from("this document")),
                hreflang: None,
                length: None,
            },
            Link {
                href: format!("{}/api", root),
                rel: Some(String::from("service-desc")),
                r#type: Some(String::from("text/yaml")),
                title: Some(String::from("the API definition")),
                hreflang: None,
                length: None,
            },
            Link {
                href: format!("{}/conformance", root),
                rel: Some(String::from("conformance")),
                r#type: Some(String::from("application/json")),
                title: Some(String::from(
                    "OGC conformance classes implemented by this API",
                )),
                hreflang: None,
                length: None,
            },
            Link {
                href: format!("{}/collections", root),
                rel: Some(String::from("data")),
                r#type: Some(String::from("application/json")),
                title: Some(String::from("Metadata about the resource collections")),
                hreflang: None,
                length: None,
            },
        ],
    });

    match body {
        Ok(content) => {
            let mut res = Response::new(200);
            res.set_body(content);
            Ok(res)
        }
        Err(_) => Ok(Response::new(500)),
    }
}

async fn handle_api(_: Request<()>) -> tide::Result {
    let mut res = Response::new(200);
    res.set_body(fs::read_to_string("api/ogcapi-features-1.yaml")?);
    Ok(res)
}

async fn handle_conformance(_: Request<()>) -> tide::Result {
    let conformance = Conformance {
        conforms_to: vec![
            String::from("http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core"),
            String::from("http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30"),
            String::from("http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/html"),
            String::from("http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson"),
        ],
    };
    let body = serde_json::to_string_pretty(&conformance);

    match body {
        Ok(content) => {
            let mut res = Response::new(200);
            res.set_body(content);
            Ok(res)
        }
        Err(_) => Ok(Response::new(500)),
    }
}

async fn handle_collections(req: Request<()>) -> tide::Result {
    let mut res = Response::new(200);
    res.set_body(fs::read_to_string("api/ogcapi-features-1.yaml")?);
    Ok(res)
}

fn main() -> tide::Result<()> {
    // parse openapi definition
    let openapi_path = std::path::Path::new("api/ogcapi-features-1.yaml");
    let openapi_string = &fs::read_to_string(openapi_path).expect("Read openapi file to string");
    let openapi: OpenAPI =
        serde_yaml::from_str(openapi_string).expect("Deserialize openapi string");

    // serve
    task::block_on(async {
        let mut app = tide::new();

        app.at("/").get(handle_root);
        app.at("/api").get(handle_api);
        app.at("/conformance").get(handle_conformance);
        app.at("/collections").get(handle_collections);

        app.at("/submit").post(|mut req: Request<()>| async move {
            let cat: Cat = req.body_json().await?;
            println!("cat name: {}", cat.name);

            let cat = Cat {
                name: "chashu".into(),
            };

            let mut res = Response::new(200);
            res.set_body(Body::from_json(&cat)?);
            Ok(res)
        });

        app.at("/animals").get(|_| async {
            Ok(json!({
                "meta": { "count": 2 },
                "animals": [
                    { "type": "cat", "name": "chashu" },
                    { "type": "cat", "name": "nori" }
                ]
            }))
        });
        let server: &Server = &openapi.servers[0];
        app.listen(server.url.clone()).await?;
        Ok(())
    })
}
