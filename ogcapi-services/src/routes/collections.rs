use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    headers::HeaderMap,
    http::{header::LOCATION, StatusCode},
    Json,
    {routing::get, Router},
};

// use serde::Deserialize;
// use serde_with::{serde_as, DisplayFromStr};
use url::Position;

use ogcapi_types::common::{
    link_rel::{DATA, ITEMS, SELF},
    media_type::{GEO_JSON, JSON},
    Collection, Collections, Crs, Link,
};

use crate::{extractors::RemoteUrl, Result, State};

const CONFORMANCE: [&str; 3] = [
    "http://www.opengis.net/spec/ogcapi-common-1/1.0/req/core",
    "http://www.opengis.net/spec/ogcapi-common-2/1.0/req/collections",
    "http://www.opengis.net/spec/ogcapi_common-2/1.0/req/json",
];

async fn collections(
    // Query(query): Query<CollectionQuery>,
    RemoteUrl(url): RemoteUrl,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<Collections>> {
    let mut collections = state.drivers.collections.list_collections().await?;

    let base = &url[..Position::AfterPath];

    collections.collections.iter_mut().for_each(|c| {
        c.links.append(&mut vec![
            Link::new(format!("{}/{}", base, c.id), SELF),
            Link::new(format!("{}/{}/items", base, c.id), ITEMS)
                .mime(GEO_JSON)
                .title(format!("Items of {}", c.title.as_ref().unwrap_or(&c.id))),
        ]);
    });

    collections.links = vec![Link::new(url, SELF).mime(JSON).title("this document")];
    collections.crs = vec![Crs::default(), Crs::from(4326)];

    Ok(Json(collections))
}

/// Create new collection metadata
async fn create(
    Json(collection): Json<Collection>,
    Extension(state): Extension<Arc<State>>,
) -> Result<(StatusCode, HeaderMap)> {
    let location = state
        .drivers
        .collections
        .create_collection(&collection)
        .await?;
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, location.parse().unwrap());
    Ok((StatusCode::CREATED, headers))
}

/// Get collection metadata
async fn read(
    Path(collection_id): Path<String>,
    RemoteUrl(url): RemoteUrl,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<Collection>> {
    let mut collection = state
        .drivers
        .collections
        .read_collection(&collection_id)
        .await?;

    collection.links.push(
        Link::new(format!("{}/items", &url[..Position::AfterPath]), SELF)
            .mime(GEO_JSON)
            .title(format!(
                "Items of {}",
                collection.title.as_ref().unwrap_or(&collection.id)
            )),
    );

    Ok(Json(collection))
}

/// Update collection metadata
async fn update(
    Path(collection_id): Path<String>,
    Json(mut collection): Json<Collection>,
    Extension(state): Extension<Arc<State>>,
) -> Result<StatusCode> {
    collection.id = collection_id;

    state
        .drivers
        .collections
        .update_collection(&collection)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Delete collection metadata
async fn remove(
    Path(collection_id): Path<String>,
    Extension(state): Extension<Arc<State>>,
) -> Result<StatusCode> {
    state
        .drivers
        .collections
        .delete_collection(&collection_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub(crate) fn router(state: &State) -> Router {
    let mut root = state.root.write().unwrap();
    root.links.push(
        Link::new(format!("{}/collections", &state.remote), DATA)
            .title("Metadata about the resource collections")
            .mime(JSON),
    );

    let mut conformance = state.conformance.write().unwrap();
    conformance
        .conforms_to
        .append(&mut CONFORMANCE.map(String::from).to_vec());

    Router::new()
        .route("/collections", get(collections).post(create))
        .route(
            "/collections/:collection_id",
            get(read).put(update).delete(remove),
        )
}
