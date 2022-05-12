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
    link_rel::{DATA, ITEMS, PARENT, ROOT, SELF},
    media_type::{GEO_JSON, JSON},
    Collection, Collections, Crs, Link,
};

use crate::{extractors::RemoteUrl, Result, State};

const CONFORMANCE: [&str; 3] = [
    "http://www.opengis.net/spec/ogcapi-common-1/1.0/req/core",
    "http://www.opengis.net/spec/ogcapi-common-2/1.0/req/collections",
    "http://www.opengis.net/spec/ogcapi_common-2/1.0/req/json",
];

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

    collection.links.extend_from_slice(&[
        Link::new(&url[..Position::BeforePath], ROOT),
        Link::new(&url[..Position::BeforePath], PARENT),
        Link::new(&url, SELF),
        Link::new(&url.join("items")?[..Position::AfterPath], ITEMS).mediatype(GEO_JSON),
        Link::new(&url.join("location")?[..Position::AfterPath], DATA)
            .title("EDR location query endpoint"),
    ]);

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

async fn collections(
    // Query(query): Query<CollectionQuery>,
    RemoteUrl(url): RemoteUrl,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<Collections>> {
    let mut collections = state.drivers.collections.list_collections().await?;

    let base = &url[..Position::AfterPath];

    collections.collections.iter_mut().for_each(|c| {
        c.links.append(&mut vec![
            Link::new(&url[..Position::BeforePath], ROOT).mediatype(JSON),
            Link::new(format!("{}/{}", base, c.id), SELF).mediatype(JSON),
            Link::new(format!("{}/{}/items", base, c.id), ITEMS).mediatype(GEO_JSON),
            Link::new(format!("{}/{}/location", base, c.id), DATA)
                .title("EDR location query endpoint"),
        ]);
    });

    collections.links = vec![Link::new(&url, SELF).mediatype(JSON).title("this document")];
    collections.crs = vec![Crs::default(), Crs::from(4326)];

    Ok(Json(collections))
}

pub(crate) fn router(state: &State) -> Router {
    let mut root = state.root.write().unwrap();
    root.links.push(
        Link::new(format!("{}/collections", &state.remote), DATA)
            .title("Metadata about the resource collections")
            .mediatype(JSON),
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
