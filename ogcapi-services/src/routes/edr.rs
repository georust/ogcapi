use std::convert::TryInto;

use axum::{
    extract::{Extension, Path, Query},
    response::Headers,
    routing::get,
    Json, Router,
};
use chrono::Utc;
use url::Position;

use crate::extractors::RemoteUrl;
use crate::{Result, State};
use ogcapi_entities::common::{Link, MediaType};
use ogcapi_entities::edr::EdrQuery;
use ogcapi_entities::features::{Feature, FeatureCollection};

const CONFORMANCE: [&str; 3] = [
    "http://www.opengis.net/spec/ogcapi-edr-1/1.0/conf/core",
    "http://www.opengis.net/spec/ogcapi-edr-1/1.0/conf/oas30",
    "http://www.opengis.net/spec/ogcapi-edr-1/1.0/conf/geojson",
];

async fn query(
    RemoteUrl(url): RemoteUrl,
    Path(collection_id): Path<String>,
    Query(query): Query<EdrQuery>,
    Extension(state): Extension<State>,
) -> Result<(
    Headers<Vec<(&'static str, String)>>,
    Json<FeatureCollection>,
)> {
    tracing::debug!("{:#?}", &query);

    let srid: i32 = query.crs.clone().try_into().unwrap();
    let storage_srid = state.db.storage_srid(&collection_id).await?;

    let mut geometry_type = query.coords.split('(').next().unwrap().to_uppercase();
    geometry_type.retain(|c| !c.is_whitespace());

    let spatial_predicate = match url.path_segments().unwrap().into_iter().last().unwrap() {
        "position" | "area" | "trajectory" => {
            if geometry_type.ends_with('Z') || geometry_type.ends_with('M') {
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
                &query.within.unwrap_or_else(|| "0".to_string()),
                &query.within_units.unwrap_or_else(|| "m".to_string())
            );
            tracing::debug!("Line: {}", line);
            let distance = rink_core::one_line(&mut ctx, &line)
                .ok()
                .and_then(|s| s.split(' ').next().and_then(|s| s.parse::<f64>().ok()))
                .expect("Failed to parse & convert distance");

            if geometry_type.ends_with('Z') || geometry_type.ends_with('M') {
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
            let bbox: Vec<&str> = query.coords.split(',').collect();
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
                .split(',')
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
        collection_id, properties, spatial_predicate
    )];

    let number_matched = sqlx::query(sql.join(" ").as_str())
        .bind(srid)
        .execute(&state.db.pool)
        .await?
        .rows_affected();

    let mut features: Vec<Feature> = sqlx::query_as(sql.join(" ").as_str())
        .bind(&srid)
        .fetch_all(&state.db.pool)
        .await?;

    for feature in features.iter_mut() {
        feature.links = Some(sqlx::types::Json(vec![Link::new(&format!(
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
        links: None,
        time_stamp: Some(Utc::now().to_string()),
        number_matched: Some(number_matched),
        number_returned: Some(number_returned),
    };

    let headers = Headers(vec![
        ("Content-Crs", query.crs.to_string()),
        ("Content-Type", MediaType::GeoJSON.to_string()),
    ]);

    Ok((headers, Json(feature_collection)))
}

// async fn instances() {}

// async fn instance() {}

pub fn router(state: &State) -> Router {
    let mut conformance = state.conformance.write().unwrap();
    conformance
        .conforms_to
        .append(&mut CONFORMANCE.map(String::from).to_vec());

    Router::new().route("/edr/:collectionId/:query_type", get(query))
    // .route("/collections/:collectionId/instances", get(instances))
    // .route("/collections/:collectionId/instances/:id", get(instance))
}
