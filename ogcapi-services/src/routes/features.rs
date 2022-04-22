use std::convert::TryInto;

use anyhow::Context;
use axum::extract::{Extension, Path};
use axum::{
    http::{
        header::{CONTENT_TYPE, LOCATION},
        HeaderMap, StatusCode,
    },
    routing::get,
    Json, Router,
};
use chrono::Utc;
use sqlx::PgPool;
use url::Position;

use crate::extractors::{Qs, RemoteUrl};
use crate::{Error, Result, State};
use ogcapi_types::common::{Bbox, Crs, Link, LinkRel, MediaType};
use ogcapi_types::features::{Feature, FeatureCollection, Query};

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
    headers.insert(LOCATION, location.parse().unwrap());
    Ok((StatusCode::CREATED, headers))
}

async fn read(
    Path((collection_id, id)): Path<(String, String)>,
    Qs(query): Qs<Query>,
    RemoteUrl(url): RemoteUrl,
    Extension(state): Extension<State>,
) -> Result<(HeaderMap, Json<Feature>)> {
    let crs = query.crs.clone().unwrap_or_default();

    is_supported_crs(&collection_id, &crs, &state.db.pool).await?;

    let srid = crs.clone().try_into().ok();
    let mut feature = state.db.select_feature(&collection_id, &id, srid).await?;

    feature.links = vec![
        Link::new(&url, LinkRel::default()).mime(MediaType::GeoJSON),
        Link::new(url.join(".")?, LinkRel::Collection).mime(MediaType::GeoJSON),
    ];

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Crs",
        crs.to_string()
            .parse()
            .context("Unable to parse `Content-Crs` header value")?,
    );
    headers.insert(
        CONTENT_TYPE,
        MediaType::GeoJSON.to_string().parse().unwrap(),
    );

    Ok((headers, Json(feature)))
}

async fn update(
    Path((collection_id, id)): Path<(String, String)>,
    Json(mut feature): Json<Feature>,
    Extension(state): Extension<State>,
) -> Result<StatusCode> {
    feature.id = Some(id);
    feature.collection = Some(collection_id);

    state.db.update_feature(&feature).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn remove(
    Path((collection_id, id)): Path<(String, String)>,
    Extension(state): Extension<State>,
) -> Result<StatusCode> {
    state.db.delete_feature(&collection_id, &id).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn items(
    Path(collection_id): Path<String>,
    Qs(mut query): Qs<Query>,
    RemoteUrl(mut url): RemoteUrl,
    Extension(state): Extension<State>,
) -> Result<(HeaderMap, Json<FeatureCollection>)> {
    tracing::debug!("{:#?}", query);

    let crs = query.crs.clone().unwrap_or_default();

    is_supported_crs(&collection_id, &crs, &state.db.pool).await?;

    let srid: Option<i32> = crs.clone().try_into().ok();

    let mut sql = vec![format!(
        r#"
        SELECT
            id,
            type,
            properties,
            ST_AsGeoJSON(ST_Transform(geom, $1))::jsonb as geometry,
            links,
            '{0}' as collection
        FROM items.{0}
        "#,
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

    let mut links = vec![Link::new(&url, LinkRel::default()).mime(MediaType::GeoJSON)];

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
                url.set_query(serde_qs::to_string(&query).ok().as_deref());
                let previous = Link::new(&url, LinkRel::Prev).mime(MediaType::GeoJSON);
                links.push(previous);
            }

            if !(offset + limit) as u64 >= number_matched {
                query.offset = Some(offset + limit);
                url.set_query(serde_qs::to_string(&query).ok().as_deref());
                let next = Link::new(&url, LinkRel::Next).mime(MediaType::GeoJSON);
                links.push(next);
            }
        }
    }

    let mut features: sqlx::types::Json<Vec<Feature>> = sqlx::query_scalar(&format!(
        r#"
        SELECT array_to_json(array_agg(row_to_json(t)))
        FROM ( {} ) t
        "#,
        sql.join(" ")
    ))
    .bind(&srid)
    .fetch_one(&state.db.pool)
    .await?;

    for feature in features.iter_mut() {
        feature.links = vec![Link::new(
            format!(
                "{}/{}",
                &url[..Position::AfterPath],
                feature.id.as_ref().unwrap()
            ),
            LinkRel::default(),
        )
        .mime(MediaType::GeoJSON)]
    }

    let number_returned = features.len();

    let feature_collection = FeatureCollection {
        r#type: "FeatureCollection".to_string(),
        features: features.0,
        links,
        time_stamp: Some(Utc::now().to_rfc3339()),
        number_matched: Some(number_matched),
        number_returned: Some(number_returned),
    };

    let mut headers = HeaderMap::new();
    headers.insert("Content-Crs", crs.to_string().parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        MediaType::GeoJSON.to_string().parse().unwrap(),
    );

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
