use openapiv3::OpenAPI;
use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use tide::http::{mime::JSON, url::Position, Mime};
use tide::{After, Body, Request, Response, Result, StatusCode};

use oapi::schema::{
    Collection, Collections, Conformance, Exception, Extent, FeatureCollection, LandingPage, Link,
    SpatialExtent,
};

#[derive(Clone)]
struct Service {
    api: OpenAPI,
}

impl Service {
    fn from_file(filepath: &str) -> Service {
        let path = Path::new(filepath);
        let content = &fs::read(path).expect("Read openapi file to string");
        if let Some(extension) = path.extension() {
            let openapi = match extension.to_str() {
                Some("yaml") => {
                    serde_yaml::from_slice(content).expect("Deserialize openapi string")
                }
                Some("json") => {
                    serde_json::from_slice(content).expect("Deserialize openapi string")
                }
                _ => panic!("Unable to read API definition from '{}'", filepath),
            };
            Service { api: openapi }
        } else {
            panic!("Unable to read API definition from '{}'", filepath)
        }
    }
}

async fn handle_root(req: Request<Service>) -> Result {
    let root = req.url();
    let info = req.state().api.info.clone();
    let landing_page = LandingPage {
        title: Some(info.title),
        description: info.description,
        links: vec![
            Link {
                href: format!("{}", root),
                rel: Some(String::from("self")),
                r#type: Some(String::from("application/json")),
                title: Some(String::from("this document")),
                ..Default::default()
            },
            Link {
                href: format!("{}api", root),
                rel: Some(String::from("service-desc")),
                r#type: Some(String::from("application/vnd.oai.openapi+json;version=3.0")),
                title: Some(String::from("the API definition")),
                ..Default::default()
            },
            Link {
                href: format!("{}conformance", root),
                rel: Some(String::from("conformance")),
                r#type: Some(String::from("application/json")),
                title: Some(String::from(
                    "OGC conformance classes implemented by this API",
                )),
                ..Default::default()
            },
            Link {
                href: format!("{}collections", root),
                rel: Some(String::from("data")),
                r#type: Some(String::from("application/json")),
                title: Some(String::from("Metadata about the resource collections")),
                ..Default::default()
            },
        ],
    };

    let mut res = Response::new(200);
    res.set_content_type(JSON);
    res.set_body(Body::from_json(&landing_page)?);
    Ok(res)
}

async fn handle_api(req: Request<Service>) -> Result {
    let mut res = Response::new(200);
    res.set_content_type(Mime::from_str("application/vnd.oai.openapi+json;version=3.0").unwrap());
    res.set_body(Body::from_json(&req.state().api)?);
    Ok(res)
}

async fn handle_conformance(_: Request<Service>) -> Result {
    let conformance = Conformance {
        conforms_to: vec![
            String::from("http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core"),
            String::from("http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30"),
            // String::from("http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/html"),
            String::from("http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson"),
        ],
    };

    let mut res = Response::new(200);
    res.set_content_type(JSON);
    res.set_body(Body::from_json(&conformance)?);
    Ok(res)
}

async fn handle_collections(req: Request<Service>) -> Result {
    let root = req.url();
    let collections = Collections {
        links: vec![Link {
            href: format!("{}", root),
            rel: Some(String::from("self")),
            r#type: Some(String::from("application/json")),
            title: Some(String::from("this document")),
            ..Default::default()
        }],
        collections: vec![Collection {
            id: String::from("fc"),
            title: Some(String::from("Test Collection")),
            description: None,
            extent: Some(Extent {
                spatial: Some(SpatialExtent {
                    bbox: Some(vec![vec![-180.0, -90.0, 180.0, 90.0]]),
                    crs: Some(String::from("http://www.opengis.net/def/crs/OGC/1.3/CRS84")),
                }),
                temporal: None,
            }),
            item_type: Some(String::from("feature")),
            crs: Some(vec![String::from(
                "http://www.opengis.net/def/crs/OGC/1.3/CRS84",
            )]),
            links: vec![Link {
                href: format!("{}/fc/items", root),
                rel: Some(String::from("items")),
                r#type: Some(String::from("application/geo+json")),
                title: Some(String::from("Features")),
                ..Default::default()
            }],
        }],
    };

    let mut res = Response::new(200);
    res.set_content_type(JSON);
    res.set_body(Body::from_json(&collections)?);
    Ok(res)
}

async fn handle_collection(req: Request<Service>) -> Result {
    let root = req.url();
    let collecetion_id: String = req.param("collection_id").unwrap();

    if !vec!["fc"].iter().any(|&i| i == collecetion_id) {
        return Ok(Response::new(404));
    }

    let collection = Collection {
        id: String::from("fc"),
        title: Some(String::from("Test Collection")),
        description: None,
        extent: Some(Extent {
            spatial: Some(SpatialExtent {
                bbox: Some(vec![vec![-180.0, -90.0, 180.0, 90.0]]),
                crs: Some(String::from("http://www.opengis.net/def/crs/OGC/1.3/CRS84")),
            }),
            temporal: None,
        }),
        item_type: Some(String::from("feature")),
        crs: Some(vec![String::from(
            "http://www.opengis.net/def/crs/OGC/1.3/CRS84",
        )]),
        links: vec![Link {
            href: format!("{}/fc/items", root),
            rel: Some(String::from("items")),
            r#type: Some(String::from("application/geo+json")),
            title: Some(String::from("Features")),
            ..Default::default()
        }],
    };

    let mut res = Response::new(200);
    res.set_content_type(JSON);
    res.set_body(Body::from_json(&collection)?);
    Ok(res)
}

async fn handle_items(req: Request<Service>) -> Result {
    let mut res = Response::new(200);

    let url = req.url();

    let mut params: HashMap<String, String> = req.query()?;
    println!("{:#?}", params);
    let limit_string = params.remove("limit");
    let bbox_string = params.remove("bbox");
    let datetime_string = params.remove("datetime");

    if params.len() > 0 {
        return Ok(Response::new(400));
    }

    let mut limit: u32 = 10;
    if let Some(number) = limit_string {
        match number.parse::<u32>() {
            Err(_) => return Ok(Response::new(400)),
            Ok(number) => limit = number,
        }
    };

    let collecetion_id: String = req.param("collection_id").unwrap();
    println!("Collection ID: {:#?}", collecetion_id);

    if !vec!["fc"].iter().any(|&i| i == collecetion_id) {
        return Ok(Response::new(404));
    }
    let feature_collection = FeatureCollection {
        r#type: String::from("FeatureCollection"),
        features: vec![],
        links: Some(vec![Link {
            href: format!("{}", &url[..Position::AfterPath]),
            rel: Some(String::from("self")),
            r#type: Some(String::from("application/geo+json")),
            ..Default::default()
        }]),
        ..Default::default()
    };

    let mut res = Response::new(200);
    res.set_content_type(JSON);
    res.set_body(Body::from_json(&feature_collection)?);
    Ok(res)
}

async fn exception(result: Result) -> Result {
    let mut res = result.unwrap_or_else(|e| Response::new(e.status()));

    if res.status().is_success() {
        return Ok(res);
    }

    let exception = match res.status() {
        StatusCode::BadRequest => Exception {
            code: res.status().to_string(),
            description: Some(String::from("A query parameter has an invalid value.")),
        },
        StatusCode::NotFound => Exception {
            code: res.status().to_string(),
            description: Some(String::from("The requested URI was not found.")),
        },
        StatusCode::InternalServerError => Exception {
            code: res.status().to_string(),
            description: Some(String::from("A server error occurred.")),
        },
        _ => Exception {
            code: res.status().to_string(),
            description: None,
        },
    };

    res.set_content_type(JSON);
    res.set_body(Body::from_json(&exception)?);
    Ok(res)
}

#[async_std::main]
async fn main() -> Result<()> {
    // parse openapi definition
    let service = Service::from_file("api/ogcapi-features-1.yaml");

    // serve
    tide::log::start();

    let mut app = tide::with_state(service.clone());

    app.middleware(After(exception));

    app.at("/").get(handle_root);
    app.at("/api").get(handle_api);
    app.at("/conformance").get(handle_conformance);
    app.at("/collections").get(handle_collections);
    app.at("/collections/:collection_id").get(handle_collection);
    app.at("/collections/:collection_id/items")
        .get(handle_items);

    let server = &service.api.servers[0];
    let address = server.url.replace("http://", "");
    app.listen(address).await?;
    Ok(())
}
