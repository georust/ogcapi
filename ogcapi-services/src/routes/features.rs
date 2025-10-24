use anyhow::Context;
use axum::{
    Json,
    extract::{Path, State},
    http::{
        HeaderMap, StatusCode,
        header::{CONTENT_TYPE, LOCATION},
    },
};
use utoipa_axum::{router::OpenApiRouter, routes};

use ogcapi_types::{
    common::{
        Collection, Crs, Exception, Link, Linked,
        link_rel::{COLLECTION, NEXT, PREV, ROOT, SELF},
        media_type::{GEO_JSON, JSON},
    },
    features::{Feature, FeatureCollection, FeatureId, Query},
};

use crate::{
    AppState, Error, Result,
    extractors::{Qs, RemoteUrl},
};

const CONFORMANCE: [&str; 3] = [
    "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core",
    // "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30",
    "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson",
    "http://www.opengis.net/spec/ogcapi-features-2/1.0/conf/crs",
];

/// Create new item
#[utoipa::path(post, path = "/collections/{collectionId}/items", tag = "Data",
    params(
        ("collectionId" = String, Path, description = "local identifier of a collection")
    ),
    request_body = Feature,
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
    State(state): State<AppState>,
    RemoteUrl(url): RemoteUrl,
    Path(collection_id): Path<String>,
    Json(mut feature): Json<Feature>,
) -> Result<(StatusCode, HeaderMap)> {
    feature.collection = Some(collection_id);

    let id = state.drivers.features.create_feature(&feature).await?;

    let location = url.join(&format!("items/{id}"))?;

    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, location.as_str().parse().unwrap());

    Ok((StatusCode::CREATED, headers))
}

/// Fetch a single feature
///
/// Fetch the feature with id `featureId` in the feature collection with id
/// `collectionId`.
#[utoipa::path(get, path = "/collections/{collectionId}/items/{featureId}", tag = "Data", 
    params(
        ("collectionId" = String, Path, description = "local identifier of a collection"),
        ("featureId" = String, Path, description = "local identifier of a feature")
    ),
    responses(
        (
            status = 200,
            description = "fetch the feature with id `featureId` in the feature \
            collection with id `collectionId`", 
            body = Feature),
        (
            status = 404, description = "The requested resource does not exist \
            on the server. For example, a path parameter had an incorrect value.", 
            body = Exception, example = json!(Exception::new_from_status(404))
        ),
        (
            status = 500, description = "A server error occurred.", 
            body = Exception, example = json!(Exception::new_from_status(500))
        )
    )
)]
async fn read(
    State(state): State<AppState>,
    RemoteUrl(url): RemoteUrl,
    Path((collection_id, id)): Path<(String, String)>,
    Qs(query): Qs<Query>,
) -> Result<(HeaderMap, Json<Feature>)> {
    let collection = state
        .drivers
        .collections
        .read_collection(&collection_id)
        .await?
        .ok_or(Error::NotFound)?;
    is_supported_crs(&collection, &query.crs).await?;

    let mut feature = state
        .drivers
        .features
        .read_feature(&collection_id, &id, &query.crs)
        .await?
        .ok_or(Error::NotFound)?;

    feature.links.insert_or_update(&[
        Link::new(&url, SELF).mediatype(GEO_JSON),
        Link::new(url.join("../../..")?, ROOT).mediatype(JSON),
        Link::new(url.join(&format!("../../{collection_id}"))?, COLLECTION).mediatype(JSON),
    ]);
    feature.links.resolve_relative_links();

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Crs",
        format!("<{}>", query.crs)
            .parse()
            .context("Unable to parse `Content-Crs` header value")?,
    );
    headers.insert(CONTENT_TYPE, GEO_JSON.parse().unwrap());

    Ok((headers, Json(feature)))
}

/// Update collection item
#[utoipa::path(put, path = "/collections/{collectionId}/items/{featureId}", tag = "Data",
    params(
        ("collectionId" = String, Path, description = "local identifier of a collection"),
        ("featureId" = String, Path, description = "local identifier of a feature")
    ),
    request_body = Feature,
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
    Path((collection_id, id)): Path<(String, String)>,
    Json(mut feature): Json<Feature>,
) -> Result<StatusCode> {
    match feature.id {
        Some(ref fid) => assert_eq!(id, fid.to_string()),
        None => feature.id = Some(FeatureId::String(id)),
    }

    feature.collection = Some(collection_id);

    state.drivers.features.update_feature(&feature).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Delete collection item
#[utoipa::path(delete, path = "/collections/{collectionId}/items/{featureId}", tag = "Data",
    params(
        ("collectionId" = String, Path, description = "local identifier of a collection"),
        ("featureId" = String, Path, description = "local identifier of a feature")
    ),
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
    State(state): State<AppState>,
    Path((collection_id, id)): Path<(String, String)>,
) -> Result<StatusCode> {
    state
        .drivers
        .features
        .delete_feature(&collection_id, &id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Fetch features
///
/// Fetch features of the feature collection with id `collectionId`.
///
/// Every feature in a dataset belongs to a collection. A dataset may consist
/// of multiple feature collections. A feature collection is often a collection
/// of features of a similar type, based on a common schema.
#[utoipa::path(get, path = "/collections/{collectionId}/items", tag = "Data", 
    params(
        ("collectionId" = String, Path, description = "local identifier of a collection"),
        Query,
    ),
    responses(
        (
            status = 200,
            description = "The response is a document consisting of features \
            in the collection. The features included in the response are \
            determined by the server based on the query parameters of the \
            request. To support access to larger collections without \
            overloading the client, the API supports paged access with links \
            to the next page, if more features are selected that the page size. \
            \n \
            The `bbox` and `datetime` parameter can be used to select only a \
            subset of the features in the collection (the features that are in \
            the bounding box or time interval). The `bbox` parameter matches \
            all features in the collection that are not associated with a \
            location, too. The `datetime` parameter matches all features in the \
            collection that are not associated with a time stamp or interval, too. \
            \n \
            The `limit` parameter may be used to control the subset of the selected \
            features that should be returned in the response, the page size. Each \
            page may include information about the number of selected and returned \
            features (`numberMatched` and `numberReturned`) as well as links to \
            support paging (link relation next).", 
            body = FeatureCollection),
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
async fn items(
    State(state): State<AppState>,
    RemoteUrl(mut url): RemoteUrl,
    Path(collection_id): Path<String>,
    Qs(mut query): Qs<Query>,
) -> Result<(HeaderMap, Json<FeatureCollection>)> {
    tracing::debug!("{:#?}", query);

    // Limit
    if let Some(limit) = query.limit {
        if limit > 10000 {
            query.limit = Some(10000);
        }
    } else {
        query.limit = Some(100);
    }

    let collection = state
        .drivers
        .collections
        .read_collection(&collection_id)
        .await?
        .ok_or(Error::NotFound)?;
    is_supported_crs(&collection, &query.crs).await?;

    // TODO: validate additional parameters

    let mut fc = state
        .drivers
        .features
        .list_items(&collection_id, &query)
        .await?;

    fc.links.insert_or_update(&[
        Link::new(&url, SELF).mediatype(GEO_JSON),
        Link::new(url.join("../..")?, ROOT).mediatype(JSON),
        Link::new(url.join(".")?, COLLECTION).mediatype(JSON),
    ]);

    // pagination
    if let Some(limit) = query.limit {
        if query.offset.is_none() {
            query.offset = Some(0);
        }

        if let Some(offset) = query.offset {
            if offset != 0 && offset >= limit {
                query.offset = Some(offset - limit);
                url.set_query(serde_qs::to_string(&query).ok().as_deref());
                let previous = Link::new(&url, PREV).mediatype(GEO_JSON);
                fc.links.insert_or_update(&[previous]);
            }

            if let Some(number_matched) = fc.number_matched
                && number_matched > (offset + limit) as u64
            {
                query.offset = Some(offset + limit);
                url.set_query(serde_qs::to_string(&query).ok().as_deref());
                let next = Link::new(&url, NEXT).mediatype(GEO_JSON);
                fc.links.insert_or_update(&[next]);
            }
        }
    }

    for feature in fc.features.iter_mut() {
        feature.links.insert_or_update(&[
            Link::new(
                url.join(&format!("items/{}", feature.id.as_ref().unwrap()))?,
                SELF,
            )
            .mediatype(GEO_JSON),
            Link::new(url.join("../..")?, ROOT).mediatype(JSON),
            Link::new(url.join(&format!("../{}", collection.id))?, COLLECTION).mediatype(JSON),
        ])
    }

    let mut headers = HeaderMap::new();
    headers.insert("Content-Crs", format!("<{}>", query.crs).parse().unwrap());
    headers.insert(CONTENT_TYPE, GEO_JSON.parse().unwrap());

    Ok((headers, Json(fc)))
}

async fn is_supported_crs(collection: &Collection, crs: &Crs) -> Result<(), Error> {
    if collection.crs.contains(crs) {
        Ok(())
    } else {
        Err(Error::ApiException(
            (StatusCode::BAD_REQUEST, format!("Unsuported CRS `{crs}`")).into(),
        ))
    }
}

pub(crate) fn router(state: &AppState) -> OpenApiRouter<AppState> {
    state.conformance.write().unwrap().extend(&CONFORMANCE);

    OpenApiRouter::new()
        .routes(routes!(items, create))
        .routes(routes!(read, update, remove))
}
