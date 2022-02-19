use std::convert::TryInto;

use chrono::{SecondsFormat, Utc};
use sqlx::types::Json;
use sqlx::PgPool;
use tide::{Body, Request, Response, Result, Server};
use url::Position;

use crate::common::{
    core::{Bbox, Exception, Link, LinkRel, MediaType},
    crs::Crs,
};
use crate::features::{Feature, FeatureCollection, Query};
use crate::server::State;

const CONFORMANCE: [&str; 4] = [
    "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core",
    "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30",
    "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson",
    "http://www.opengis.net/spec/ogcapi-features-2/1.0/conf/crs",
];

async fn insert(mut req: Request<State>) -> tide::Result {
    let mut feature: Feature = req.body_json().await?;

    feature.collection = Some(req.param("collectionId")?.to_owned());

    let location = req.state().db.insert_feature(&feature).await?;

    let mut res = Response::new(201);
    res.insert_header("Location", location);
    Ok(res)
}

async fn get(req: Request<State>) -> tide::Result {
    let id: i64 = req.param("id")?.parse()?;
    let collection = req.param("collectionId")?;

    let query: Query = req.query()?;

    let crs = query.crs.clone().unwrap_or_default();
    if let Some(res) = validate_crs(collection, &crs, &req.state().db.pool).await {
        return Ok(res);
    }

    let srid = crs.clone().try_into().ok();
    let mut feature = req.state().db.select_feature(collection, &id, srid).await?;

    let url = req.url();
    feature.links = Some(Json(vec![
        Link::new(url.as_str()).mime(MediaType::GeoJSON),
        Link::new(&format!(
            "{}/collections/{}",
            &url[..Position::BeforePath],
            collection
        ))
        .mime(MediaType::GeoJSON),
    ]));

    let mut res = Response::new(200);
    res.insert_header("Content-Crs", crs.to_string());
    res.set_content_type(MediaType::GeoJSON);
    res.set_body(Body::from_json(&feature)?);
    Ok(res)
}

async fn update(mut req: Request<State>) -> tide::Result {
    let id: i64 = req.param("id")?.parse()?;
    let collection = req.param("collectionId")?.to_owned();

    let mut feature: Feature = req.body_json().await?;

    feature.id = Some(id);
    feature.collection = Some(collection.to_string());

    req.state().db.update_feature(&feature).await?;

    Ok(Response::new(204))
}

async fn delete(req: Request<State>) -> tide::Result {
    let id: i64 = req.param("id")?.parse()?;
    let collection = req.param("collectionId")?;

    req.state().db.delete_feature(collection, &id).await?;

    Ok(Response::new(204))
}

async fn items(req: Request<State>) -> Result {
    let mut url = req.url().to_owned();

    let collection: &str = req.param("collectionId")?;

    let mut query: Query = req.query()?;
    tide::log::debug!("{:#?}", query);

    let crs = query.crs.clone().unwrap_or_default();
    if let Some(res) = validate_crs(collection, &crs, &req.state().db.pool).await {
        return Ok(res);
    }

    let srid: Option<i32> = crs.clone().try_into().ok();

    let mut sql = vec![format!(
        "SELECT
            id,
            feature_type,
            properties,
            ST_AsGeoJSON(ST_Transform(geom, $1))::jsonb as geometry,
            links,
            stac_version,
            stac_extensions,
            ST_AsGeoJSON(ST_Transform(geom, $1), 9, 1)::jsonb -> 'bbox' as bbox,
            assets,
            '{0}' as collection
        FROM items.{0}",
        collection
    )];

    if let Some(bbox) = query.bbox.as_ref() {
        let crs = query.bbox_crs.clone().unwrap_or_default();
        if let Some(res) = validate_crs(collection, &crs, &req.state().db.pool).await {
            return Ok(res);
        }

        let storage_srid = req.state().db.storage_srid(collection).await?;

        let bbox_srid: i32 = crs.try_into().unwrap();
        let envelope = match bbox {
            Bbox::Bbox2D(bbox) => format!(
                "ST_MakeEnvelope({}, {}, {}, {}, {})",
                bbox[0], bbox[1], bbox[2], bbox[3], bbox_srid
            ),
            Bbox::Bbox3D(bbox) => format!(
                "ST_MakeEnvelope({}, {}, {}, {}, {})",
                bbox[0], bbox[1], bbox[3], bbox[4], bbox_srid
            ),
        };
        sql.push(format!(
            "WHERE geom && ST_Transform({}, {})",
            envelope, storage_srid
        ));
    }

    let number_matched = sqlx::query(sql.join(" ").as_str())
        .bind(srid)
        .execute(&req.state().db.pool)
        .await?
        .rows_affected();

    let mut links = vec![Link::new(url.as_str()).mime(MediaType::GeoJSON)];

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
                query.offset = Some(offset - limit);
                url.set_query(Some(&query.to_string()));
                let previous = Link::new(url.as_str())
                    .relation(LinkRel::Prev)
                    .mime(MediaType::GeoJSON);
                links.push(previous);
            }

            if !(offset + limit) as u64 >= number_matched {
                query.offset = Some(offset + limit);
                url.set_query(Some(&query.to_string()));
                let next = Link::new(url.as_str())
                    .relation(LinkRel::Next)
                    .mime(MediaType::GeoJSON);
                links.push(next);
            }
        }
    }

    let mut features: Vec<Feature> = sqlx::query_as(sql.join(" ").as_str())
        .bind(&srid)
        .fetch_all(&req.state().db.pool)
        .await?;

    for feature in features.iter_mut() {
        feature.links = Some(Json(vec![Link::new(&format!(
            "{}/{}",
            &url[..Position::AfterPath],
            feature.id.as_ref().unwrap()
        ))
        .mime(MediaType::GeoJSON)]))
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
    res.set_content_type(MediaType::GeoJSON);
    res.set_body(Body::from_json(&feature_collection)?);
    Ok(res)
}

async fn validate_crs(collection: &str, crs: &Crs, pool: &PgPool) -> Option<Response> {
    if sqlx::query("SELECT id FROM meta.collections WHERE id = $1 AND collection->'crs' ? $2")
        .bind(&collection)
        .bind(crs.to_string())
        .execute(pool)
        .await
        .unwrap()
        .rows_affected()
        == 0
    {
        let mut res = Response::new(400);
        res.set_body(
            Body::from_json(&Exception {
                r#type: "https://httpwg.org/specs/rfc7231.html#status.400".to_string(),
                status: Some(400),
                detail: Some("Unsupported crs".to_string()),
                ..Default::default()
            })
            .unwrap(),
        );

        Some(res)
    } else {
        None
    }
}

pub(crate) async fn register(app: &mut Server<State>) {
    app.state()
        .conformance
        .write()
        .await
        .conforms_to
        .append(&mut CONFORMANCE.map(String::from).to_vec());

    app.at("/collections/:collectionId/items")
        .get(items)
        .post(insert);
    app.at("/collections/:collectionId/items/:id")
        .get(get)
        .put(update)
        .delete(delete);
}
