use chrono::{SecondsFormat, Utc};
use serde::Deserialize;
use sqlx::types::Json;
use tide::http::{url::Position, Method};
use tide::{Body, Request, Response, Result};

use crate::common::Exception;
use crate::common::{
    crs,
    crs::CRS,
    link::{ContentType, Link, LinkRelation},
};
use crate::features::schema::{Collection, Collections, Feature, FeatureCollection};
use crate::features::service::State;

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub limit: Option<isize>,
    pub offset: Option<isize>,
    pub bbox: Option<String>,
    pub datetime: Option<String>,
    pub crs: Option<CRS>,
}

impl Query {
    fn to_string(&self) -> String {
        let mut query_str = vec![];
        if let Some(limit) = self.limit {
            query_str.push(format!("limit={}", limit));
        }
        if let Some(offset) = self.offset {
            query_str.push(format!("offset={}", offset));
        }
        if let Some(bbox) = &self.bbox {
            query_str.push(format!("bbox={}", bbox));
        }
        if let Some(datetime) = &self.datetime {
            query_str.push(format!("datetime={}", datetime));
        }
        query_str.join("&")
    }

    fn to_string_with_offset(&self, offset: isize) -> String {
        let mut new_query = self.clone();
        new_query.offset = Some(offset);
        new_query.to_string()
    }
}

pub async fn handle_root(req: Request<State>) -> Result {
    let url = req.url();

    let mut landing_page = req.state().root.clone();
    for link in landing_page.links.iter_mut() {
        link.href = format!("{}{}", url, link.href.trim_matches('/'));
    }

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&landing_page)?);
    Ok(res)
}

pub async fn handle_api(req: Request<State>) -> Result {
    let mut res = Response::new(200);
    res.set_content_type(ContentType::OPENAPI);
    res.set_body(Body::from_json(&req.state().openapi)?);
    Ok(res)
}

pub async fn handle_conformance(req: Request<State>) -> Result {
    let mut res = Response::new(200);
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
            r#type: Some(ContentType::GEOJSON),
            title: Some(format!(
                "Items of {}",
                collection.title.clone().unwrap_or(collection.id.clone())
            )),
            ..Default::default()
        });
        collection.links.push(link);

        // set default item type
        if collection.item_type.is_none() {
            collection.item_type = Some("feature".to_string());
        }
        // handle default crs
        match &collection.crs {
            Some(crs) => crs.to_owned().push("#/crs".to_string()),
            None => collection.crs = Some(vec!["#/crs".to_owned()]),
        }
    }

    let collections = Collections {
        links: vec![Link {
            href: url.to_string(),
            r#type: Some(ContentType::JSON),
            title: Some("this document".to_string()),
            ..Default::default()
        }],
        crs: vec![crs::EPSG_WGS84.to_owned(), crs::EPSG_4979.to_owned()],
        collections,
    };

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&collections)?);
    Ok(res)
}

pub async fn handle_collection(mut req: Request<State>) -> Result {
    let url = req.url();
    let method = req.method();

    let id: Option<String> = if method != Method::Post {
        Some(req.param("collection")?)
    } else {
        None
    };

    let mut res = Response::new(200);
    let mut collection: Collection;

    match method {
        Method::Get => {
            collection = sqlx::query_as("SELECT * FROM meta.collections WHERE id = $1")
                .bind(id)
                .fetch_one(&req.state().pool)
                .await?;

            let link = Json(Link {
                href: format!("{}/items", &url[..Position::AfterPath]),
                rel: LinkRelation::Items,
                r#type: Some(ContentType::GEOJSON),
                title: collection.title.clone(),
                ..Default::default()
            });
            collection.links.push(link);

            // set default item type
            if collection.item_type.is_none() {
                collection.item_type = Some("feature".to_string());
            }
            // handle default crs
            match &collection.crs {
                Some(crs) => {
                    let mut crs = crs.to_owned();
                    crs.push(crs::EPSG_WGS84.to_string());
                    crs.push(crs::EPSG_4979.to_string());
                    collection.crs = Some(crs);
                }
                None => {
                    collection.crs =
                        Some(vec![crs::EPSG_WGS84.to_owned(), crs::EPSG_4979.to_owned()])
                }
            }
        }
        Method::Post | Method::Put => {
            collection = req.body_json().await?;

            let mut sql = if method == Method::Post {
                vec![
                    "INSERT INTO meta.collections",
                    "(id, title, description, links, extent, item_type, crs, storage_crs, storage_crs_coordinate_epoche)",
                    "VALUES ($1, $2, $3, $4, $5, $6, $7)",
                ]
            } else {
                vec![
                    "UPDATE meta.collections",
                    "SET title = $2, description = $3, links = $4, extent = $5, item_type = $6, crs = $7, storage_crs = $8, storage_crs_coordinate_epoche = $9)",
                    "WHERE id = $1",
                ]
            };
            sql.push("RETURNING id, title, description, links, extent, item_type, crs, storage_crs, storage_crs_coordinate_epoche");

            let mut tx = req.state().pool.begin().await?;
            collection = sqlx::query_as(&sql.join(" ").as_str())
                .bind(&collection.id)
                .bind(&collection.title)
                .bind(&collection.description)
                .bind(&collection.links)
                .bind(&collection.extent)
                .bind(&collection.item_type)
                .bind(&collection.crs)
                .bind(&collection.storage_crs)
                .bind(&collection.storage_crs_coordinate_epoch)
                .fetch_one(&mut tx)
                .await?;
            tx.commit().await?;
        }
        Method::Delete => {
            let mut tx = req.state().pool.begin().await?;
            let _deleted = sqlx::query("DELETE FROM meta.collections WHERE id = $1")
                .bind(id)
                .execute(&mut tx)
                .await?;
            tx.commit().await?;

            return Ok(res);
        }
        _ => unimplemented!(),
    }
    res.set_body(Body::from_json(&collection)?);
    Ok(res)
}

pub async fn handle_items(req: Request<State>) -> Result {
    let mut url = req.url().to_owned();

    let collection: String = req.param("collection")?;

    let mut query: Query = req.query()?;

    let mut links = vec![Link {
        href: url.to_string(),
        r#type: Some(ContentType::GEOJSON),
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

    // pagination
    if let Some(limit) = query.limit {
        sql.push("ORDER BY id".to_string());
        sql.push(format!("LIMIT {}", limit));

        if query.offset.is_none() {
            query.offset = Some(0);
        }

        if let Some(offset) = query.offset {
            sql.push(format!("OFFSET {}", offset));

            if offset != 0 && offset >= limit {
                url.set_query(Some(&query.to_string_with_offset(offset - limit)));
                let previous = Link {
                    href: url.to_string(),
                    rel: LinkRelation::Previous,
                    r#type: Some(ContentType::GEOJSON),
                    ..Default::default()
                };
                links.push(previous);
            }

            if !(offset + limit) as u64 >= number_matched {
                url.set_query(Some(&query.to_string_with_offset(offset + limit)));
                let next = Link {
                    href: url.to_string(),
                    rel: LinkRelation::Next,
                    r#type: Some(ContentType::GEOJSON),
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
    res.set_content_type(ContentType::GEOJSON);
    res.set_body(Body::from_json(&feature_collection)?);
    Ok(res)
}

pub async fn handle_item(mut req: Request<State>) -> Result {
    let url = req.url().clone();
    let method = req.method();

    let id: Option<String> = if method != Method::Post {
        Some(req.param("id")?)
    } else {
        None
    };

    let collection: String = req.param("collection")?;

    let mut res = Response::new(200);
    let mut feature: Feature;

    match method {
        Method::Get => {
            let sql = r#"
            SELECT id, type, ST_AsGeoJSON(geometry)::jsonb as geometry, properties, links
            FROM data.features
            WHERE collection = $1 AND id = $2
            "#;
            feature = sqlx::query_as(sql)
                .bind(collection)
                .bind(&id)
                .fetch_one(&req.state().pool)
                .await?;
        }
        Method::Post | Method::Put => {
            feature = req.body_json().await?;

            let mut sql = if method == Method::Post {
                vec![
                    "INSERT INTO data.features",
                    "(id, type, properties, geometry, links)",
                    "VALUES ($1, $2, $3, $4, $5)",
                ]
            } else {
                vec![
                    "UPDATE data.features",
                    "SET type = $2, properties = $3, geometry = $4, links = $5)",
                    "WHERE id = $1",
                ]
            };
            sql.push(
                "RETURNING id, type, properties, ST_AsGeoJSON(geometry)::jsonb as geometry, links",
            );

            let mut tx = req.state().pool.begin().await?;
            feature = sqlx::query_as(&sql.join(" ").as_str())
                .bind(&feature.id)
                .bind(&feature.r#type)
                .bind(&feature.properties)
                .bind(&feature.geometry)
                .bind(&feature.links)
                .fetch_one(&mut tx)
                .await?;
            tx.commit().await?;
        }
        Method::Delete => {
            let mut tx = req.state().pool.begin().await?;

            let _deleted = sqlx::query("DELETE FROM data.features WHERE id = $1")
                .bind(id)
                .execute(&mut tx)
                .await?;

            tx.commit().await?;

            return Ok(res);
        }
        _ => unimplemented!(),
    }

    feature.links = Some(Json(vec![
        Link {
            href: url.to_string(),
            r#type: Some(ContentType::GEOJSON),
            ..Default::default()
        },
        Link {
            href: url.as_str().replace(&format!("/items/{}", id.unwrap()), ""),
            rel: LinkRelation::Collection,
            r#type: Some(ContentType::GEOJSON),
            ..Default::default()
        },
    ]));

    res.set_content_type(ContentType::GEOJSON);
    res.set_body(Body::from_json(&feature)?);
    Ok(res)
}

pub async fn exception(result: Result) -> Result {
    match result {
        Ok(res) => {
            if res.status().is_success() {
                Ok(res)
            } else {
                println!("WTF:\n{:#?}", res);
                panic!()
            }
        }
        Err(err) => {
            let status = err.status();
            let mut res = Response::new(status);
            let exception = Exception {
                code: status.to_string(),
                description: Some(err.to_string()),
            };
            res.set_body(Body::from_json(&exception)?);
            Ok(res)
        }
    }
}

pub async fn handle_favicon(_: Request<State>) -> Result {
    let mut res = Response::new(200);
    res.set_body(Body::from_file("favicon.ico").await?);
    Ok(res)
}
