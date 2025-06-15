use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header::LOCATION},
};
use hyper::HeaderMap;
use utoipa_axum::{router::OpenApiRouter, routes};

use ogcapi_types::common::{
    Collection, Collections, Crs, Exception, Link, Linked, Query,
    link_rel::{DATA, ITEMS, ROOT, SELF},
    media_type::{GEO_JSON, JSON},
};

use crate::{
    AppState, Error, Result,
    extractors::{Qs, RemoteUrl},
};

const CONFORMANCE: [&str; 4] = [
    "http://www.opengis.net/spec/ogcapi-common-1/1.0/conf/core",
    "http://www.opengis.net/spec/ogcapi-common-1/1.0/conf/landingPage",
    // "http://www.opengis.net/spec/ogcapi-common-1/1.0/conf/oas30",
    // "http://www.opengis.net/spec/ogcapi-common-1/1.0/conf/html",
    "http://www.opengis.net/spec/ogcapi_common-2/1.0/conf/json",
    "http://www.opengis.net/spec/ogcapi-common-2/1.0/conf/collections",
];

/// Create new collection metadata
#[utoipa::path(post, path = "/collections", tag = "Collections", 
    request_body = Collection,
    responses(
        (
            status = 201, description = "Created.", 
            headers(
                ("Location", description = "URI of the newly added resource.")
            )
        ),
        (
            status = 409, description = "Already exists.", 
            body = Exception, example = json!(Exception::new_from_status(409))
        ),
        (
            status = 500, description = "A server error occurred.", 
            body = Exception, example = json!(Exception::new_from_status(500))
        )
    )
)]
async fn create(
    RemoteUrl(url): RemoteUrl,
    State(state): State<AppState>,
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
///
/// Describe the feature collection with id `collectionId`
#[utoipa::path(get, path = "/collections/{collectionId}", tag = "Collections", 
    params(
        ("collectionId" = String, Path, description = "local identifier of a collection")
    ),
    responses(
        (
            status = 200,
            description = "Information about the feature collection with id \
                `collectionId`. \
                \n\nThe response contains a link to the items in the collection \
                (path `/collections/{collectionId}/items`, link relation `items`) \
                as well as key information about the collection. This \
                information includes: \
                \n* A local identifier for the collection that is unique for \
                the dataset; \
                \n* A list of coordinate reference systems (CRS) in which \
                geometries may be returned by the server. The first CRS is \
                the default coordinate reference system (the default is always \
                WGS 84 with axis order longitude/latitude); \
                \n* An optional title and description for the collection; \
                \n* An optional extent that can be used to provide an indication \
                of the spatial and temporal extent of the collection - typically \
                derived from the data; \
                \n* An optional indicator about the type of the items in the \
                collection (the default value, if the indicator is not provided, \
                is 'feature').", 
            body = Collection),
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
async fn read(
    RemoteUrl(url): RemoteUrl,
    State(state): State<AppState>,
    Path(collection_id): Path<String>,
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
#[utoipa::path(put, path = "/collections/{collectionId}", tag = "Collections",
    params(
        ("collectionId" = String, Path, description = "local identifier of a collection")
    ),
    responses(
        (status = 204, description = "Successfuly updataed, no content."),
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
#[utoipa::path(delete, path = "/collections/{collectionId}", tag = "Collections",
    params(
        ("collectionId" = String, Path, description = "local identifier of a collection")
    ),
    request_body = Collection,
    responses(
        (status = 204, description = "Successfuly deleted, no content."),
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

/// The feature collections in the dataset.
#[utoipa::path(get, path = "/collections", tag = "Capabilities", 
    responses(
        (
            status = 200,
            description = "The feature collections shared by this API. \
            \n\nThe dataset is organized as one or more feature collections. \
            This resource provides information about and access to the \
            collections. \
            \n\nThe response contains the list of collections. For each \
            collection, a link to the items in the collection (path \
            `/collections/{collectionId}/items`, link relation `items`) as \
            well as key information about the collection. This information \
            includes: \
            \n* A local identifier for the collection that is unique for \
            the dataset; \
            \n* A list of coordinate reference systems (CRS) in which \
            geometries may be returned by the server. The first CRS is the \
            default coordinate reference system (the default is always WGS 84 \
            with axis order longitude/latitude); \
            \n* An optional title and description for the collection; \
            \n* An optional extent that can be used to provide an indication \
            of the spatial and temporal extent of the collection - typically \
            derived from the data; \
            \n* An optional indicator about the type of the items in the \
            collection (the default value, if the indicator is not provided, \
            is 'feature').", 
            body = Collections
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

pub(crate) fn router(state: &AppState) -> OpenApiRouter<AppState> {
    let mut root = state.root.write().unwrap();
    root.links.push(
        Link::new("collections", DATA)
            .title("Metadata about the resource collections")
            .mediatype(JSON),
    );

    state.conformance.write().unwrap().extend(&CONFORMANCE);

    OpenApiRouter::new()
        .routes(routes!(collections, create))
        .routes(routes!(read, update, remove))
}
