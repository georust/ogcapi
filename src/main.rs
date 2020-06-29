use chrono::{SecondsFormat, Utc};
use openapiv3::OpenAPI;
use serde::{Deserialize, Serialize};
use serde_yaml;
use sqlx::postgres::PgPool;
use sqlx::types::Json;
use std::fs::File;
use std::str::FromStr;
use tide::http::{mime, url::Position, Mime, Url};
use tide::{After, Body, Request, Response, Result, StatusCode};

use ogcapi::schema::{
    Collection, Collections, Conformance, Exception, Feature, FeatureCollection, LandingPage, Link,
    Query,
};

static GEOJSON: &str = "application/geo+json";

struct State {
    api: OpenAPI,
    config: Config,
    pool: PgPool,
}

#[derive(Serialize, Deserialize)]
struct Config {
    conformance: Conformance,
    root_links: Vec<Link>,
}

impl State {
    async fn new(api: &str, db_url: &str) -> State {
        let config = File::open("Config.json").expect("Open config");
        let config: Config = serde_json::from_reader(config).expect("Deserialize config");
        let api = File::open(api).expect("Open api file");
        let api: OpenAPI = serde_yaml::from_reader(api).expect("Deserialize api document");
        let pool = PgPool::new(db_url).await.expect("Create pg pool");
        State { api, config, pool }
    }
}

async fn handle_root(req: Request<State>) -> Result {
    let url = req.url();

    let info = req.state().api.info.clone();
    let mut links = req.state().config.root_links.clone();
    for link in links.iter_mut() {
        link.href = format!("{}{}", url, link.href.trim_matches('/'));
    }

    let landing_page = LandingPage {
        title: Some(info.title),
        description: info.description,
        links,
    };

    let mut res = Response::new(200);
    res.set_content_type(mime::JSON);
    res.set_body(Body::from_json(&landing_page)?);
    Ok(res)
}

async fn handle_api(req: Request<State>) -> Result {
    let mut res = Response::new(200);
    res.set_content_type(Mime::from_str("application/vnd.oai.openapi+json;version=3.0").unwrap());
    res.set_body(Body::from_json(&req.state().api)?);
    Ok(res)
}

async fn handle_conformance(req: Request<State>) -> Result {
    let mut res = Response::new(200);
    res.set_content_type(mime::JSON);
    res.set_body(Body::from_json(&req.state().config.conformance)?);
    Ok(res)
}

async fn handle_collections(req: Request<State>) -> Result {
    let url = req.url();

    let mut collections: Vec<Collection> = sqlx::query_as("SELECT * FROM meta.collections")
        .fetch_all(&req.state().pool)
        .await?;

    for collection in &mut collections {
        let link = Json(Link {
            href: format!("{}/{}/items", &url[..Position::AfterPath], collection.id),
            rel: Some("items".to_string()),
            r#type: Some(GEOJSON.to_string()),
            title: collection.title.clone(),
            ..Default::default()
        });
        collection.links.push(link);
    }

    let collections = Collections {
        links: vec![Link {
            href: url[..Position::AfterPath].to_string(),
            rel: Some("self".to_string()),
            r#type: Some(mime::JSON.to_string()),
            title: Some("this document".to_string()),
            ..Default::default()
        }],
        collections,
    };

    let mut res = Response::new(200);
    res.set_content_type(mime::JSON);
    res.set_body(Body::from_json(&collections)?);
    Ok(res)
}

async fn handle_collection(req: Request<State>) -> Result {
    let url = req.url();

    let id: String = req.param("collection")?;

    let collection: Option<Collection> =
        sqlx::query_as("SELECT * FROM meta.collections WHERE id = $1")
            .bind(id)
            .fetch_optional(&req.state().pool)
            .await?;

    if let Some(mut collection) = collection {
        let link = Json(Link {
            href: format!("{}/items", &url[..Position::AfterPath]),
            rel: Some("items".to_string()),
            r#type: Some(GEOJSON.to_string()),
            title: collection.title.clone(),
            ..Default::default()
        });
        collection.links.push(link);

        let mut res = Response::new(200);
        res.set_content_type(mime::JSON);
        res.set_body(Body::from_json(&collection)?);
        Ok(res)
    } else {
        Ok(Response::new(404))
    }
}

async fn handle_items(req: Request<State>) -> Result {
    let mut url = req.url().to_owned();

    let collection: String = req.param("collection")?;

    let mut query: Query = req.query()?;

    let mut links = vec![Link {
        href: url.to_string(),
        rel: Some("self".to_string()),
        r#type: Some(GEOJSON.to_string()),
        ..Default::default()
    }];

    let mut sql = vec![
        "SELECT id, type, ST_AsGeoJSON(geometry)::jsonb as geometry, properties, links".to_string(),
        "FROM data.features".to_string(),
        "WHERE collection = $1".to_string(),
    ];

    let number_matched = sqlx::query(sql.join(" ").as_str())
        .bind(&collection)
        .execute(&req.state().pool)
        .await?;

    if let Some(limit) = query.limit {
        sql.push("ORDER BY id".to_string());
        sql.push(format!("LIMIT {}", limit));

        if query.offset.is_none() {
            query.offset = Some(0);
        }

        if let Some(offset) = query.offset {
            sql.push(format!("OFFSET {}", offset));

            if offset != 0 && offset >= limit {
                url.set_query(Some(&format!("limit={}&offset={}", limit, offset - limit)));
                let previous = Link {
                    href: url.to_string(),
                    rel: Some("previous".to_string()),
                    r#type: Some(GEOJSON.to_string()),
                    ..Default::default()
                };
                links.push(previous);
            }

            if !(offset + limit) as u64 >= number_matched {
                url.set_query(Some(&format!("limit={}&offset={}", limit, offset + limit)));
                let next = Link {
                    href: url.to_string(),
                    rel: Some("next".to_string()),
                    r#type: Some(GEOJSON.to_string()),
                    ..Default::default()
                };
                links.push(next);
            }
        }
    }

    let features: Vec<Feature> = sqlx::query_as(sql.join(" ").as_str())
        .bind(&collection)
        .fetch_all(&req.state().pool)
        .await?;

    let number_returned = features.len();

    let feature_collection = FeatureCollection {
        r#type: "FeatureCollection".to_string(),
        features,
        links: Some(links),
        time_stamp: Some(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
        number_matched: Some(number_matched),
        number_returned: Some(number_returned),
    };

    let mut res = Response::new(200);
    res.set_content_type(mime::JSON);
    res.set_body(Body::from_json(&feature_collection)?);
    Ok(res)
}

async fn handle_item(req: Request<State>) -> Result {
    let url = req.url();

    let id: String = req.param("id")?;
    let collection: String = req.param("collection")?;

    let sql = r#"
    SELECT id, type, ST_AsGeoJSON(geometry)::jsonb as geometry, properties, links
    FROM data.features
    WHERE collection = $1 AND id = $2
    "#;
    let feature: Option<Feature> = sqlx::query_as(sql)
        .bind(collection)
        .bind(&id)
        .fetch_optional(&req.state().pool)
        .await?;

    if let Some(mut feature) = feature {
        feature.links = Some(Json(vec![
            Link {
                href: url[..Position::AfterPath].to_string(),
                rel: Some("self".to_string()),
                r#type: Some(GEOJSON.to_string()),
                ..Default::default()
            },
            Link {
                href: url[..Position::AfterPath].replace(&format!("/items/{}", id), ""),
                rel: Some("collection".to_string()),
                r#type: Some(GEOJSON.to_string()),
                ..Default::default()
            },
        ]));
        let mut res = Response::new(200);
        res.set_content_type(mime::JSON);
        res.set_body(Body::from_json(&feature)?);
        Ok(res)
    } else {
        return Ok(Response::new(404));
    }
}

async fn exception(result: Result) -> Result {
    match result {
        Ok(mut res) => {
            if res.status().is_success() {
                Ok(res)
            } else {
                let exception = match res.status() {
                    StatusCode::BadRequest => Exception {
                        code: res.status().to_string(),
                        description: Some("A query parameter has an invalid value.".to_string()),
                    },
                    StatusCode::NotFound => Exception {
                        code: res.status().to_string(),
                        description: Some("The requested URI was not found.".to_string()),
                    },
                    StatusCode::InternalServerError => Exception {
                        code: res.status().to_string(),
                        description: Some("A server error occurred.".to_string()),
                    },
                    _ => Exception {
                        code: res.status().to_string(),
                        description: Some("Unknown error.".to_string()),
                    },
                };
                res.set_content_type(mime::JSON);
                res.set_body(Body::from_json(&exception)?);
                Ok(res)
            }
        }
        Err(err) => {
            let status = err.status();
            let mut res = Response::new(status);
            let exception = Exception {
                code: status.to_string(),
                description: Some(err.to_string()),
            };
            res.set_content_type(mime::JSON);
            res.set_body(Body::from_json(&exception)?);
            Ok(res)
        }
    }
}

#[async_std::main]
async fn main() -> Result<()> {
    // create state
    let state = State::new(
        "api/ogcapi-features-1.yaml",
        "postgresql://postgres:postgres@localhost/ogcapi",
    )
    .await;

    let url = Url::from_str(&state.api.servers[0].url).expect("Parse url from string");

    // serve
    tide::log::start();

    let mut app = tide::with_state(state);

    app.middleware(After(exception));

    app.at("/").get(handle_root);
    app.at("/api").get(handle_api);
    app.at("/conformance").get(handle_conformance);
    app.at("/collections").get(handle_collections);
    app.at("/collections/:collection").get(handle_collection);
    app.at("/collections/:collection/items").get(handle_items);
    app.at("/collections/:collection/items/:id")
        .get(handle_item);

    app.listen(&url[Position::BeforeHost..Position::AfterPort])
        .await?;
    Ok(())
}
