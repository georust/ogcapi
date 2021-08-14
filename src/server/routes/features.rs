use std::str::FromStr;

use chrono::{SecondsFormat, Utc};
use geojson::{Bbox, Geometry};
use sqlx::types::Json;
use tide::{Body, Request, Response, Result};

use crate::common::{ContentType, Link, LinkRelation, CRS, OGC_CRS84};
use crate::db::Db;
use crate::features::{Assets, Feature, FeatureCollection, FeatureType, Query};

pub async fn create_item(mut req: Request<Db>) -> tide::Result {
    let mut feature: Feature = req.body_json().await?;

    let collection: &str = req.param("collection")?;

    feature = sqlx::query_file_as!(
        Feature,
        "sql/feature_insert.sql",
        collection,
        feature.feature_type as _,
        feature.properties,
        feature.geometry as _,
        feature.links as _,
        feature.stac_version,
        feature.stac_extensions.as_deref(),
        feature.assets as _
    )
    .fetch_one(&req.state().pool)
    .await?;

    if let Some(links) = feature.links.as_mut() {
        links.push(Link {
            href: format!("{}/{}", req.url().as_str(), feature.id.clone().unwrap()),
            r#type: Some(ContentType::GeoJSON),
            ..Default::default()
        });
        links.push(Link {
            href: req.url().as_str().replace("/items", ""),
            rel: LinkRelation::Collection,
            r#type: Some(ContentType::GeoJSON),
            ..Default::default()
        });
    };

    let mut res = Response::new(200);
    // res.set_content_type(ContentType::GeoJSON);
    res.set_body(Body::from_json(&feature)?);
    Ok(res)
}

pub async fn read_item(req: Request<Db>) -> tide::Result {
    let id: i64 = req.param("id")?.parse()?;

    let mut query: Query = req.query()?;

    let crs = query.crs.take().unwrap_or(OGC_CRS84.to_string());
    let srid: i32 = CRS::from_str(&crs)
        .unwrap()
        .ogc_to_epsg()
        .map_or("4326".to_string(), |crs| crs.code)
        .parse()?;

    let mut feature = sqlx::query_file_as!(Feature, "sql/feature_select.sql", id, srid)
        .fetch_one(&req.state().pool)
        .await?;

    feature.links = Some(Json(vec![
        Link {
            href: req.url().to_string(),
            r#type: Some(ContentType::GeoJSON),
            ..Default::default()
        },
        Link {
            href: req
                .url()
                .as_str()
                .replace(&format!("/items/{}", feature.id.unwrap()), ""),
            rel: LinkRelation::Collection,
            r#type: Some(ContentType::GeoJSON),
            ..Default::default()
        },
    ]));

    let mut res = Response::new(200);
    res.insert_header("Content-Crs", crs.to_string());
    res.set_content_type(ContentType::GeoJSON);
    res.set_body(Body::from_json(&feature)?);
    Ok(res)
}

pub async fn update_item(mut req: Request<Db>) -> tide::Result {
    let url = req.url().clone();
    let mut feature: Feature = req.body_json().await?;

    let id: i64 = req.param("id")?.parse()?;
    let collection: &str = req.param("collection")?;

    feature = sqlx::query_file_as!(
        Feature,
        "sql/feature_update.sql",
        id,
        collection,
        feature.feature_type as _,
        feature.properties,
        feature.geometry as _,
        feature.links as _,
        feature.stac_version,
        feature.stac_extensions.as_deref(),
        feature.assets as _
    )
    .fetch_one(&req.state().pool)
    .await?;

    if let Some(links) = feature.links.as_mut() {
        links.push(Link {
            href: url.to_string(),
            r#type: Some(ContentType::GeoJSON),
            ..Default::default()
        });
        links.push(Link {
            href: url.as_str().replace(&format!("/items/{}", id), ""),
            rel: LinkRelation::Collection,
            r#type: Some(ContentType::GeoJSON),
            ..Default::default()
        });
    };

    let mut res = Response::new(200);
    // res.set_content_type(ContentType::GeoJSON);
    res.set_body(Body::from_json(&feature)?);
    Ok(res)
}

pub async fn delete_item(req: Request<Db>) -> tide::Result {
    let id: i64 = req.param("id")?.parse()?;

    sqlx::query_file_as!(Feature, "sql/feature_delete.sql", &id)
        .execute(&req.state().pool)
        .await?;

    Ok(Response::new(200))
}

pub async fn handle_items(req: Request<Db>) -> Result {
    let mut url = req.url().to_owned();

    let collection: &str = req.param("collection")?;

    let mut query: Query = req.query()?;

    let crs = query.crs.take().unwrap_or(OGC_CRS84.to_string());
    let srid: i32 = CRS::from_str(&crs)
        .unwrap()
        .ogc_to_epsg()
        .map_or("4326".to_string(), |crs| crs.code)
        .parse()?;

    let mut sql = vec!["SELECT 
            id, 
            feature_type, 
            properties, 
            ST_AsGeoJSON( ST_Transform( geom, $2::int))::jsonb as geometry, 
            links, 
            stac_version, 
            stac_extensions, 
            ST_AsGeoJSON( ST_Transform( geom, $2::int), 9, 1)::jsonb -> 'bbox' as bbox, 
            assets, 
            collection
        FROM features
        WHERE collection = $1"
        .to_string()];

    if query.bbox.is_some() {
        if let Some(envelop) = query.make_envelope() {
            sql.push(format!("AND geom && {}", envelop));
        }
    }

    let number_matched = sqlx::query(sql.join(" ").as_str())
        .bind(collection)
        .bind(srid)
        .execute(&req.state().pool)
        .await?
        .rows_affected();

    let mut links = vec![Link {
        href: url.to_string(),
        r#type: Some(ContentType::GeoJSON),
        ..Default::default()
    }];

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
                url.set_query(Some(&query.as_string_with_offset(offset - limit)));
                let previous = Link {
                    href: url.to_string(),
                    rel: LinkRelation::Previous,
                    r#type: Some(ContentType::GeoJSON),
                    ..Default::default()
                };
                links.push(previous);
            }

            if !(offset + limit) as u64 >= number_matched {
                url.set_query(Some(&query.as_string_with_offset(offset + limit)));
                let next = Link {
                    href: url.to_string(),
                    rel: LinkRelation::Next,
                    r#type: Some(ContentType::GeoJSON),
                    ..Default::default()
                };
                links.push(next);
            }
        }
    }

    let mut features: Vec<Feature> = sqlx::query_as(sql.join(" ").as_str())
        .bind(&collection)
        .bind(&srid)
        .fetch_all(&req.state().pool)
        .await?;

    for feature in features.iter_mut() {
        feature.links = Some(Json(vec![Link {
            href: format!("{}/{}", url.as_str(), feature.id.clone().unwrap()),
            r#type: Some(ContentType::GeoJSON),
            ..Default::default()
        }]))
    }

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
    res.insert_header("Content-Crs", crs.to_string());
    res.set_content_type(ContentType::GeoJSON);
    res.set_body(Body::from_json(&feature_collection)?);
    Ok(res)
}
