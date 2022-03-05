use std::convert::TryInto;

use axum::extract::{Extension, Path, Query};
use axum::http::{header::HeaderName, HeaderMap, HeaderValue, StatusCode};
use axum::response::Headers;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{SecondsFormat, Utc};
use sqlx::PgPool;
use url::{Position, Url};

use crate::common::core::{Bbox, Link, LinkRel, MediaType};
use crate::common::crs::Crs;
use crate::features::{Feature, FeatureCollection, FeaturesQuery};
use crate::server::error::Error;
use crate::server::{Result, State};

const CONFORMANCE: [&str; 4] = [
    "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core",
    "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30",
    "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson",
    "http://www.opengis.net/spec/ogcapi-features-2/1.0/conf/crs",
];

async fn insert(
    Path(collection_id): Path<String>,
    Json(mut feature): Json<Feature>,
    Extension(state): Extension<State>,
) -> Result<(StatusCode, HeaderMap)> {
    feature.collection = Some(collection_id);

    let location = state.db.insert_feature(&feature).await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("location"),
        HeaderValue::from_bytes(location.as_bytes()).unwrap(),
    );
    Ok((StatusCode::CREATED, headers))
}

async fn read(
    Path((collection_id, id)): Path<(String, i64)>,
    axum::extract::Query(query): axum::extract::Query<FeaturesQuery>,
    Extension(state): Extension<State>,
) -> Result<(Headers<Vec<(&'static str, String)>>, Json<Feature>)> {
    let crs = query.crs.clone().unwrap_or_default();

    is_supported_crs(&collection_id, &crs, &state.db.pool).await?;

    let srid = crs.clone().try_into().ok();
    let mut feature = state.db.select_feature(&collection_id, &id, srid).await?;

    // TOOD: create custom extractor
    let url = Url::parse(&format!(
        "http://localhost:8484/collections/{collection_id}/items/{id}"
    ))
    .unwrap();
    feature.links = Some(sqlx::types::Json(vec![
        Link::new(url.as_str()).mime(MediaType::GeoJSON),
        Link::new(&format!(
            "{}/collections/{}",
            &url[..Position::BeforePath],
            collection_id
        ))
        .mime(MediaType::GeoJSON),
    ]));

    let headers = Headers(vec![
        ("Content-Crs", crs.to_string()),
        ("Content-Type", MediaType::GeoJSON.to_string()),
    ]);

    Ok((headers, Json(feature)))
}

async fn update(
    Path((collection_id, id)): Path<(String, i64)>,
    Json(mut feature): Json<Feature>,
    Extension(state): Extension<State>,
) -> Result<StatusCode> {
    feature.id = Some(id);
    feature.collection = Some(collection_id);

    state.db.update_feature(&feature).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn remove(
    Path((collection_id, id)): Path<(String, i64)>,
    Extension(state): Extension<State>,
) -> Result<StatusCode> {
    state.db.delete_feature(&collection_id, &id).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn items(
    Path(collection_id): Path<String>,
    Query(mut query): Query<FeaturesQuery>,
    Extension(state): Extension<State>,
) -> Result<(
    Headers<Vec<(&'static str, String)>>,
    Json<FeatureCollection>,
)> {
    // TOOD: create custom extractor
    let mut url = Url::parse(&format!(
        "http://localhost:8484/collections/{collection_id}/items"
    ))
    .unwrap();

    tracing::debug!("{:#?}", query);

    let crs = query.crs.clone().unwrap_or_default();

    is_supported_crs(&collection_id, &crs, &state.db.pool).await?;

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
        collection_id
    )];

    if let Some(bbox) = query.bbox.as_ref() {
        let crs = query.bbox_crs.clone().unwrap_or_default();

        is_supported_crs(&collection_id, &crs, &state.db.pool).await?;

        let storage_srid = state.db.storage_srid(&collection_id).await?;

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
        .execute(&state.db.pool)
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
        links: Some(links),
        time_stamp: Some(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
        number_matched: Some(number_matched),
        number_returned: Some(number_returned),
    };

    let headers = Headers(vec![
        ("Content-Crs", crs.to_string()),
        ("Content-Type", MediaType::GeoJSON.to_string()),
    ]);

    Ok((headers, Json(feature_collection)))
}

async fn is_supported_crs(collection: &str, crs: &Crs, pool: &PgPool) -> Result<(), Error> {
    let result =
        sqlx::query("SELECT id FROM meta.collections WHERE id = $1 AND collection->'crs' ? $2")
            .bind(&collection)
            .bind(crs.to_string())
            .execute(pool)
            .await?;
    if result.rows_affected() == 0 {
        Err(Error::Exception(
            StatusCode::BAD_REQUEST,
            format!("Unsuported CRS `{}`", crs),
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn router(state: &State) -> Router {
    let mut conformance = state.conformance.write().unwrap();
    conformance
        .conforms_to
        .append(&mut CONFORMANCE.map(String::from).to_vec());

    Router::new()
        .route("/collections/:collection_id/items", get(items).post(insert))
        .route(
            "/collections/:collection_id/items/:id",
            get(read).put(update).delete(remove),
        )
}
