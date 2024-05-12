use axum::{
    extract::{Path, State},
    http::{header::LOCATION, StatusCode},
    Json,
    {routing::get, Router},
};
use hyper::HeaderMap;

use ogcapi_types::common::{
    link_rel::{DATA, ITEMS, ROOT, SELF},
    media_type::{GEO_JSON, JSON},
    Collection, Collections, Crs, Link, Linked, Query,
};

use crate::{
    extractors::{Qs, RemoteUrl},
    AppState, Error, Result,
};

const CONFORMANCE: [&str; 3] = [
    "http://www.opengis.net/spec/ogcapi-common-1/1.0/conf/core",
    "http://www.opengis.net/spec/ogcapi-common-2/1.0/conf/collections",
    "http://www.opengis.net/spec/ogcapi_common-2/1.0/conf/json",
];

/// Create new collection metadata
async fn create(
    State(state): State<AppState>,
    RemoteUrl(url): RemoteUrl,
    Json(collection): Json<Collection>,
) -> Result<(StatusCode, HeaderMap)> {
    if state
        .drivers
        .collections
        .read_collection(&collection.id)
        .await?
        .is_some()
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
    State(state): State<AppState>,
    Path(collection_id): Path<String>,
    RemoteUrl(url): RemoteUrl,
) -> Result<Json<Collection>> {
    let mut collection = state
        .drivers
        .collections
        .read_collection(&collection_id)
        .await?
        .ok_or(Error::NotFound)?;

    collection.links.insert_or_update(&[
        Link::new(&url, SELF),
        Link::new(url.join("..")?, ROOT).mediatype(JSON),
    ]);

    #[cfg(not(feature = "stac"))]
    collection.links.insert_or_update(&[Link::new(
        &url.join(&format!("{}/items", collection.id))?,
        ITEMS,
    )
    .mediatype(GEO_JSON)]);

    #[cfg(feature = "stac")]
    if collection.r#type == "Collection" {
        collection.links.insert_or_update(&[Link::new(
            url.join(&format!("{}/items", collection.id))?,
            ITEMS,
        )
        .mediatype(GEO_JSON)]);
    }

    collection.links.resolve_relative_links();

    Ok(Json(collection))
}

/// Update collection metadata
async fn update(
    State(state): State<AppState>,
    Path(collection_id): Path<String>,
    Json(mut collection): Json<Collection>,
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
    State(state): State<AppState>,
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
    State(state): State<AppState>,
) -> Result<Json<Collections>> {
    let mut collections = state.drivers.collections.list_collections(&query).await?;

    for collection in collections.collections.iter_mut() {
        collection.links.insert_or_update(&[
            Link::new(url.join(&format!("collections/{}", collection.id))?, SELF).mediatype(JSON),
            Link::new(url.join(".")?, ROOT).mediatype(JSON),
            Link::new(
                url.join(&format!("collections/{}/items", collection.id))?,
                ITEMS,
            )
            .mediatype(GEO_JSON),
        ]);

        collection.links.resolve_relative_links()
    }

    collections.links = vec![
        Link::new(&url, SELF).mediatype(JSON).title("this document"),
        Link::new(url.join(".")?, ROOT).mediatype(JSON),
    ];

    collections.crs = vec![Crs::default(), Crs::from_epsg(3857)];

    Ok(Json(collections))
}

pub(crate) fn router(state: &AppState) -> Router<AppState> {
    let mut root = state.root.write().unwrap();
    root.links.push(
        Link::new("collections", DATA)
            .title("Metadata about the resource collections")
            .mediatype(JSON),
    );

    state.conformance.write().unwrap().extend(&CONFORMANCE);

    Router::new()
        .route("/collections", get(collections).post(create))
        .route(
            "/collections/:collection_id",
            get(read).put(update).delete(remove),
        )
}
