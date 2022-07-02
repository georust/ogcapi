use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    headers::HeaderMap,
    http::header::CONTENT_TYPE,
    routing::get,
    Json, Router,
};

use ogcapi_types::{
    common::{link_rel::SELF, media_type::GEO_JSON, Link},
    edr::{Query, QueryType},
    features::FeatureCollection,
};

use crate::{
    extractors::{Qs, RemoteUrl},
    Result, State,
};

const CONFORMANCE: [&str; 8] = [
    "http://www.opengis.net/spec/ogcapi-edr-1/1.0/conf/core",
    "http://www.opengis.net/spec/ogcapi-edr-1/1.0/conf/collections",
    "http://www.opengis.net/spec/ogcapi-edr-1/1.0/conf/json",
    "http://www.opengis.net/spec/ogcapi-edr-1/1.0/conf/geojson",
    "http://www.opengis.net/spec/ogcapi-edr-1/1.0/conf/edr-geojson",
    "http://www.opengis.net/spec/ogcapi-edr-1/1.0/conf/covjson",
    // "http://www.opengis.net/spec/ogcapi-edr-1/1.0/conf/html",
    "http://www.opengis.net/spec/ogcapi-edr-1/1.0/conf/oas30",
    "http://www.opengis.net/spec/ogcapi-edr-1/1.0/conf/queries",
];

async fn query(
    Path((collection_id, query_type)): Path<(String, QueryType)>,
    Qs(query): Qs<Query>,
    RemoteUrl(url): RemoteUrl,
    Extension(state): Extension<Arc<State>>,
) -> Result<(HeaderMap, Json<FeatureCollection>)> {
    tracing::debug!("{:#?}", query);

    let mut fc = state
        .drivers
        .edr
        .query(&collection_id, &query_type, &query)
        .await?;

    for feature in fc.features.iter_mut() {
        feature.links = vec![Link::new(
            &url.join(&format!(
                "items/{}",
                feature.id.as_ref().expect("Feature should have id")
            ))?,
            SELF,
        )
        .mediatype(GEO_JSON)]
    }

    let mut headers = HeaderMap::new();
    headers.insert("Content-Crs", query.crs.to_string().parse().unwrap());
    headers.insert(CONTENT_TYPE, GEO_JSON.parse().unwrap());

    Ok((headers, Json(fc)))
}

// async fn instances() {}

// async fn instance() {}

pub(crate) fn router(state: &State) -> Router {
    state.conformance.write().unwrap().extend(&CONFORMANCE);

    Router::new().route("/collections/:collection_id/:query_type", get(query))
    // .route("/collections/:collection_id/instances", get(instances))
    // .route("/collections/:collection_id/instances/:instance_id", get(instance))
    // .route("/collections/:collection_id/instances/:instance_id/:query_type", get(instance))
}
