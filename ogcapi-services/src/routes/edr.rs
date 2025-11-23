use axum::{
    Json,
    extract::{Path, State},
    http::header::CONTENT_TYPE,
};
use hyper::HeaderMap;
use utoipa_axum::{router::OpenApiRouter, routes};

use ogcapi_types::{
    common::{Exception, Link, link_rel::SELF, media_type::GEO_JSON},
    edr::{Query, QueryType},
    features::FeatureCollection,
};

use crate::{
    AppState, Result,
    extractors::{Qs, RemoteUrl},
};

const CONFORMANCE: [&str; 6] = [
    "http://www.opengis.net/spec/ogcapi-edr-1/1.1/conf/core",
    "http://www.opengis.net/spec/ogcapi-edr-1/1.1/conf/collections",
    "http://www.opengis.net/spec/ogcapi-edr-1/1.1/conf/json",
    "http://www.opengis.net/spec/ogcapi-edr-1/1.1/conf/geojson",
    // "http://www.opengis.net/spec/ogcapi-edr-1/1.1/conf/edr-geojson",
    // "http://www.opengis.net/spec/ogcapi-edr-1/1.0/conf/covjson",
    // "http://www.opengis.net/spec/ogcapi-edr-1/1.0/conf/html",
    "http://www.opengis.net/spec/ogcapi-edr-1/1.1/conf/oas30",
    "http://www.opengis.net/spec/ogcapi-edr-1/1.0/conf/queries",
];

/// Retrieve data according to the query pattern from a collection with the unique
/// identifier `collectionId`
#[utoipa::path(get, path = "/collections/{collectionId}/{queryType}", tag = "Collection data queries", 
    params(
        ("collectionId" = String, Path, description = "local identifier of a collection"),
        (
            "queryType" = QueryType, Path, 
            description = "an identifier for a specific query pattern to retrieve \
            data from a specific collection of data"
        ),
        Query,
    ),
    responses(
        (
            status = 200,
            description = "Data ranges required to construct valid queries for \
            the choosen data collection", 
            body = FeatureCollection
        ),
        (
            status = 202, description = "Data request still being processed"
        ),
        (
            status = 308, description = "Request will take a significant time to process"
        ),
        (
            status = 400, description = "General HTTP error response.", 
            body = Exception, example = json!(Exception::new_from_status(400))
        ),
        (
            status = 500, description = "A server error occurred.", 
            body = Exception, example = json!(Exception::new_from_status(500))
        )
    )
)]
async fn query(
    Path((collection_id, query_type)): Path<(String, QueryType)>,
    Qs(query): Qs<Query>,
    RemoteUrl(url): RemoteUrl,
    State(state): State<AppState>,
) -> Result<(HeaderMap, Json<FeatureCollection>)> {
    tracing::debug!("{:#?}", query);

    let (mut fc, crs) = state
        .drivers
        .edr
        .query(&collection_id, &query_type, &query)
        .await?;

    for feature in fc.features.iter_mut() {
        feature.links = vec![
            Link::new(
                url.join(&format!(
                    "items/{}",
                    feature.id.as_ref().expect("Feature should have id")
                ))?,
                SELF,
            )
            .mediatype(GEO_JSON),
        ]
    }

    let mut headers = HeaderMap::new();
    headers.insert("Content-Crs", crs.to_string().parse().unwrap());
    headers.insert(CONTENT_TYPE, GEO_JSON.parse().unwrap());

    Ok((headers, Json(fc)))
}

// async fn instances() {}

// async fn instance() {}

pub(crate) fn router(state: &AppState) -> OpenApiRouter<AppState> {
    state.conformance.write().unwrap().extend(&CONFORMANCE);

    OpenApiRouter::new().routes(routes!(query))
    // .route("/collections/{collection_id}/instances", get(instances))
    // .route("/collections/{collection_id}/instances/{instance_id}", get(instance))
    // .route("/collections/{collection_id}/instances/{instance_id}/{query_type}", get(instance))
}
