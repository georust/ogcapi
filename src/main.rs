use async_std::task;
use openapiv3::{OpenAPI, Server};
use serde_yaml;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;
use tide::http::Mime;
use tide::{Request, Response};

use oapi::schema::*;

async fn handle_root(req: Request<()>) -> tide::Result {
    let root = req.url();
    let landing_page = LandingPage {
        title: Some(String::from("Features")),
        description: Some(String::from(
            "Access to data via a Web API that conforms to the OGC API Features specification.",
        )),
        links: vec![
            Link {
                href: format!("{}", root),
                rel: Some(String::from("self")),
                r#type: Some(String::from("application/json")),
                title: Some(String::from("this document")),
                hreflang: None,
                length: None,
            },
            Link {
                href: format!("{}api", root),
                rel: Some(String::from("service-desc")),
                r#type: Some(String::from("application/vnd.oai.openapi+json;version=3.0")),
                title: Some(String::from("the API definition")),
                hreflang: None,
                length: None,
            },
            Link {
                href: format!("{}conformance", root),
                rel: Some(String::from("conformance")),
                r#type: Some(String::from("application/json")),
                title: Some(String::from(
                    "OGC conformance classes implemented by this API",
                )),
                hreflang: None,
                length: None,
            },
            Link {
                href: format!("{}collections", root),
                rel: Some(String::from("data")),
                r#type: Some(String::from("application/json")),
                title: Some(String::from("Metadata about the resource collections")),
                hreflang: None,
                length: None,
            },
        ],
    };

    let content = serde_json::to_string_pretty(&landing_page);
    match content {
        Ok(content) => {
            let mut res = Response::new(200);
            res.set_content_type(Mime::from_str("application/json").unwrap());
            res.set_body(content);
            Ok(res)
        }
        Err(_) => Ok(Response::new(500)),
    }
}

async fn handle_api(_: Request<()>) -> tide::Result {
    let mut res = Response::new(200);
    res.set_content_type(Mime::from_str("application/vnd.oai.openapi+json;version=3.0").unwrap());
    res.set_body(fs::read_to_string("api/ogcapi-features-1.json")?);
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
            res.set_content_type(Mime::from_str("application/json").unwrap());
            res.set_body(content);
            Ok(res)
        }
        Err(_) => Ok(Response::new(500)),
    }
}

async fn handle_collections(req: Request<()>) -> tide::Result {
    println!("{:#?}", req);
    let mut res = Response::new(200);
    // TODO!
    res.set_body(fs::read_to_string("api/ogcapi-features-1.yaml")?);
    Ok(res)
}

fn main() -> tide::Result<()> {
    // parse openapi definition
    let openapi_path = Path::new("api/ogcapi-features-1.yaml");
    let openapi_string = &fs::read_to_string(openapi_path).expect("Read openapi file to string");
    let openapi: OpenAPI =
        serde_yaml::from_str(openapi_string).expect("Deserialize openapi string");
    let mut file = File::create(openapi_path.with_extension("json")).unwrap();
    file.write_all(&serde_json::to_vec_pretty(&openapi).unwrap())
        .unwrap();

    // serve
    task::block_on(async {
        tide::log::start();

        let mut app = tide::new();

        app.at("/").get(handle_root);
        app.at("/api").get(handle_api);
        app.at("/conformance").get(handle_conformance);
        app.at("/collections").get(handle_collections);

        let server: &Server = &openapi.servers[0];
        let address = server.url.replace("http://", "");
        app.listen(address).await?;
        Ok(())
    })
}
