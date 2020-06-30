use chrono::{SecondsFormat, Utc};
use serde::Deserialize;
use sqlx::types::Json;
use std::str::FromStr;
use tide::http::{mime, url::Position, Mime};
use tide::{Body, Request, Response, Result, StatusCode};

use crate::common::{LinkRelation, ContentType, Link};

use crate::features::schema::{Collection, Collections, Exception, Feature, FeatureCollection};
use crate::features::service::State;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Query {
    #[serde(default)]
    pub limit: Option<isize>,
    #[serde(default)]
    pub offset: Option<isize>,
    pub bbox: Option<String>,
    pub datetime: Option<String>,
}

pub async fn handle_root(req: Request<State>) -> Result {
    let url = req.url();

    let mut landing_page = req.state().root.clone();
    for link in landing_page.links.iter_mut() {
        link.href = format!("{}{}", url, link.href.trim_matches('/'));
    }

    let mut res = Response::new(200);
    res.set_content_type(mime::JSON);
    res.set_body(Body::from_json(&landing_page)?);
    Ok(res)
}

pub async fn handle_api(req: Request<State>) -> Result {
    let mut res = Response::new(200);
    res.set_content_type(Mime::from_str("application/vnd.oai.openapi+json;version=3.0").unwrap());
    res.set_body(Body::from_json(&req.state().openapi)?);
    Ok(res)
}

pub async fn handle_conformance(req: Request<State>) -> Result {
    let mut res = Response::new(200);
    res.set_content_type(mime::JSON);
    res.set_body(Body::from_json(&req.state().conformance)?);
    Ok(res)
}

pub async fn handle_collections(req: Request<State>) -> Result {
    let url = req.url();

    let mut collections: Vec<Collection> = sqlx::query_as("SELECT * FROM meta.collections")
        .fetch_all(&req.state().pool)
        .await?;

    for collection in &mut collections {
        let link = Json(Link {
            href: format!("{}/{}/items", &url[..Position::AfterPath], collection.id),
            rel: LinkRelation::Items,
            r#type: Some(ContentType::GeoJson),
            title: collection.title.clone(),
            ..Default::default()
        });
        collection.links.push(link);
    }

    let collections = Collections {
        links: vec![Link {
            href: url.to_string(),
            r#type: Some(ContentType::Json),
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

pub async fn handle_collection(req: Request<State>) -> Result {
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
            rel: LinkRelation::Items,
            r#type: Some(ContentType::GeoJson),
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

pub async fn handle_items(req: Request<State>) -> Result {
    let mut url = req.url().to_owned();

    let collection: String = req.param("collection")?;

    let mut query: Query = req.query()?;

    let mut links = vec![Link {
        href: url.to_string(),
        r#type: Some(ContentType::GeoJson),
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
                    rel: LinkRelation::Previous,
                    r#type: Some(ContentType::GeoJson),
                    ..Default::default()
                };
                links.push(previous);
            }

            if !(offset + limit) as u64 >= number_matched {
                url.set_query(Some(&format!("limit={}&offset={}", limit, offset + limit)));
                let next = Link {
                    href: url.to_string(),
                    rel: LinkRelation::Next,
                    r#type: Some(ContentType::GeoJson),
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

pub async fn handle_item(req: Request<State>) -> Result {
    let url = req.url();

    let id: String = req.param("id")?;
    let collection: String = req.param("collection")?;

    let sql = r#"
    SELECT id, type, ST_AsGeoJSON(geometry)::jsonb as geometry, properties, links
    FROM data.features
    WHERE collection = $1 AND id = $2
    "#;
    let mut feature: Feature = sqlx::query_as(sql)
        .bind(collection)
        .bind(&id)
        .fetch_one(&req.state().pool)
        .await?;

    feature.links = Some(Json(vec![
        Link {
            href: url.to_string(),
            r#type: Some(ContentType::GeoJson),
            ..Default::default()
        },
        Link {
            href: url.as_str().replace(&format!("/items/{}", id), ""),
            rel: LinkRelation::Collection,
            r#type: Some(ContentType::GeoJson),
            ..Default::default()
        },
    ]));
    let mut res = Response::new(200);
    res.set_content_type(mime::JSON);
    res.set_body(Body::from_json(&feature)?);
    Ok(res)
}

pub async fn exception(result: Result) -> Result {
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
