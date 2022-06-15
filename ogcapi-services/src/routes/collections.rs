use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    headers::HeaderMap,
    http::{header::LOCATION, StatusCode},
    Json,
    {routing::get, Router},
};

use ogcapi_types::common::{
    link_rel::{DATA, ITEMS, PARENT, ROOT, SELF},
    media_type::{GEO_JSON, JSON},
    Collection, Collections, Crs, Link, Linked, Query,
};

use crate::{
    extractors::{Qs, RemoteUrl},
    Error, Result, State,
};

const CONFORMANCE: [&str; 3] = [
    "http://www.opengis.net/spec/ogcapi-common-1/1.0/req/core",
    "http://www.opengis.net/spec/ogcapi-common-2/1.0/req/collections",
    "http://www.opengis.net/spec/ogcapi_common-2/1.0/req/json",
];

/// Create new collection metadata
async fn create(
    Json(collection): Json<Collection>,
    RemoteUrl(url): RemoteUrl,
    Extension(state): Extension<Arc<State>>,
) -> Result<(StatusCode, HeaderMap)> {
    if state
        .drivers
        .collections
        .read_collection(&collection.id)
        .await
        .is_ok()
    {
        return Err(Error::Exception(
            StatusCode::CONFLICT,
            format!("Collection with id `{}` already exists.", collection.id),
        ));
    }

    let id = state
        .drivers
        .collections
        .create_collection(&collection)
        .await?;

    let location = url.join(&format!("collections/{id}"))?;

    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, location.as_str().parse().unwrap());

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

    collection.links.insert_or_update(&[
        Link::new(&url, SELF),
        Link::new(&url.join("..")?, PARENT).mediatype(JSON),
        Link::new(&url.join("..")?, ROOT).mediatype(JSON),
        Link::new(&url.join(&format!("{}/items", collection.id))?, ITEMS).mediatype(GEO_JSON),
        // Link::new(&url.join(&format!("{}/location", collection.id))?, DATA)
        //     .title("EDR location query endpoint"),
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
    Qs(query): Qs<Query>,
    RemoteUrl(url): RemoteUrl,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<Collections>> {
    let mut collections = state.drivers.collections.list_collections(&query).await?;

    for collection in collections.collections.iter_mut() {
        collection.links.insert_or_update(&[
            Link::new(&url.join(&format!("collections/{}", collection.id))?, SELF).mediatype(JSON),
            Link::new(&url.join(".")?, ROOT).mediatype(JSON),
            Link::new(
                &url.join(&format!("collections/{}/items", collection.id))?,
                ITEMS,
            )
            .mediatype(GEO_JSON),
            // Link::new(
            //     &url.join(&format!("collections/{}/location", collection.id))?,
            //     DATA,
            // )
            // .title("EDR location query endpoint"),
        ]);
    }

    collections.links = vec![
        Link::new(&url, SELF).mediatype(JSON).title("this document"),
        Link::new(&url.join(".")?, ROOT).mediatype(JSON),
    ];
    collections.crs = vec![Crs::default(), Crs::from_epsg(3857)];

    Ok(Json(collections))
}

pub(crate) fn router(state: &State) -> Router {
    let mut root = state.root.write().unwrap();
    root.links.push(
        Link::new("collections", DATA)
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
