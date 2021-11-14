use std::convert::TryInto;

use chrono::{SecondsFormat, Utc};
use sqlx::types::Json;
use tide::{Body, Request, Response, Result, Server};
use url::{Position, Url};

use crate::common::core::{Link, MediaType};
use crate::edr::Query;
use crate::features::{Feature, FeatureCollection};
use crate::server::State;

async fn query(req: Request<State>) -> Result {
    let collection = req.param("collectionId")?;

    let query: Query = req.query()?;
    tide::log::debug!("{:#?}", &query);

    let srid: i32 = query.crs.clone().try_into().unwrap();
    let storage_srid = req.state().db.storage_srid(&collection).await?;

    let mut geometry_type = query.coords.split("(").next().unwrap().to_uppercase();
    geometry_type.retain(|c| !c.is_whitespace());

    let spatial_predicate = match req.url().path_segments().unwrap().last().unwrap() {
        "position" | "area" | "trajectory" => {
            if geometry_type.ends_with("Z") || geometry_type.ends_with("M") {
                format!(
                    "ST_3DIntersects(geom, ST_Transform(ST_GeomFromEWKT('SRID={};{}'), {}))",
                    srid, query.coords, storage_srid
                )
            } else {
                format!(
                    "ST_Intersects(geom, ST_Transform(ST_GeomFromEWKT('SRID={};{}'), {}))",
                    srid, query.coords, storage_srid
                )
            }
        }
        "radius" => {
            let mut ctx = rink_core::simple_context().unwrap();
            let line = format!(
                "{} {} -> m",
                &query.within.unwrap_or("0".to_string()),
                &query.within_units.unwrap_or("m".to_string())
            );
            tide::log::debug!("Line: {}", line);
            let distance = rink_core::one_line(&mut ctx, &line)
                .ok()
                .and_then(|s| s.split(" ").next().and_then(|s| s.parse::<f64>().ok()))
                .expect("Failed to parse & convert distance");

            if geometry_type.ends_with("Z") || geometry_type.ends_with("M") {
                format!(
                    "ST_3DDWithin(geom, ST_Transform(ST_GeomFromEWKT('SRID={};{}'), {}))",
                    srid, query.coords, storage_srid
                )
            } else {
                format!(
                    "ST_DWithin(ST_Transform(geom, 4326)::geography, ST_Transform(ST_GeomFromEWKT('SRID={};{}'), 4326)::geography, {}, false)",
                    srid, query.coords, distance
                )
            }
        }
        "cube" => {
            let bbox: Vec<&str> = query.coords.split(",").collect();
            if bbox.len() == 4 {
                format!(
                    "ST_Intersects(geom, ST_Transform(ST_MakeEnvelope({}, {}), {})",
                    query.coords, srid, storage_srid
                )
            } else {
                format!(
                    "ST_3DIntersects(
                        geom,
                        ST_Transform(
                            ST_SetSRID(
                                ST_3DMakeBox(ST_MakePoint({}, {}, {}), ST_MakePoint({} , {}, {})),
                                {}
                            ),
                            {}
                        )
                    )",
                    bbox[0], bbox[1], bbox[2], bbox[3], bbox[4], bbox[5], srid, storage_srid
                )
            }
        }
        "corridor" => todo!(),
        _ => unimplemented!(),
    };

    let properties = if let Some(parameters) = query.parameter_name {
        format!(
            "{0} as properties",
            parameters
                .split(",")
                .map(|s| format!(
                    "('{{\"{0}\":' || (properties -> '{0}')::text || '}}')::jsonb",
                    s
                ))
                .collect::<Vec<String>>()
                .join("||")
        )
    } else {
        "properties".to_string()
    };

    let sql = vec![format!(
        "SELECT
            id,
            feature_type,
            {1},
            ST_AsGeoJSON(ST_Transform(geom, $1))::jsonb as geometry,
            links,
            stac_version,
            stac_extensions,
            ST_AsGeoJSON(ST_Transform(geom, $1), 9, 1)::jsonb -> 'bbox' as bbox,
            assets,
            '{0}' as collection
        FROM items.{0}
        WHERE {2}",
        collection, properties, spatial_predicate
    )];

    let number_matched = sqlx::query(sql.join(" ").as_str())
        .bind(srid)
        .execute(&req.state().db.pool)
        .await?
        .rows_affected();

    let mut features: Vec<Feature> = sqlx::query_as(sql.join(" ").as_str())
        .bind(&srid)
        .fetch_all(&req.state().db.pool)
        .await?;

    for feature in features.iter_mut() {
        feature.links = Some(Json(vec![Link::new(
            Url::parse(&format!(
                "{}/{}",
                &req.url()[..Position::AfterPath],
                feature.id.as_ref().unwrap()
            ))
            .unwrap(),
        )
        .mime(MediaType::GeoJSON)]))
    }

    let number_returned = features.len();

    let feature_collection = FeatureCollection {
        r#type: "FeatureCollection".to_string(),
        features,
        links: None,
        time_stamp: Some(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
        number_matched: Some(number_matched),
        number_returned: Some(number_returned),
    };

    let mut res = Response::new(200);
    res.insert_header("Content-Crs", query.crs.to_string());
    res.set_content_type(MediaType::GeoJSON);
    res.set_body(Body::from_json(&feature_collection)?);
    Ok(res)
}

async fn instances(_req: Request<State>) -> Result {
    let res = Response::new(200);
    Ok(res)
}

async fn instance(_req: Request<State>) -> Result {
    let res = Response::new(200);
    Ok(res)
}

pub(crate) fn register(app: &mut Server<State>) {
    app.at("/collections/:collectionId/position").get(query);
    app.at("/collections/:collectionId/radius").get(query);
    app.at("/collections/:collectionId/area").get(query);
    app.at("/collections/:collectionId/cube").get(query);
    app.at("/collections/:collectionId/trajectory").get(query);
    app.at("/collections/:collectionId/corridor").get(query);

    app.at("/collections/:collectionId/instances")
        .get(instances);
    app.at("/collections/:collectionId/instances/:id")
        .get(instance);
}
