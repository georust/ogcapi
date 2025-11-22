use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use hyper::header::CONTENT_TYPE;
use url::Url;
use utoipa_axum::{router::OpenApiRouter, routes};

use ogcapi_types::{
    common::{
        Bbox, Exception, Link, Linked,
        link_rel::{COLLECTION, NEXT, PREV, ROOT, SELF},
        media_type::{GEO_JSON, JSON},
    },
    features::FeatureCollection,
    stac::{SearchBody, SearchParams},
};

use crate::{
    AppState, Error, Result,
    extractors::{Qs, RemoteUrl},
};

/// Search STAC items with simple filtering.
///
/// Retrieve Items matching filters. Intended as a shorthand API for simple
/// queries.
///
/// This method is required to implement.
///
/// If this endpoint is implemented on a server, it is required to add a
/// link referring to this endpoint with `rel` set to `search` to the
/// `links` array in `GET /`. As `GET` is the default method, the `method`
/// may not be set explicitly in the link.
#[utoipa::path(get, path = "/search", tag = "Item Search", 
    params(SearchParams),
    responses(
        (
            status = 200,
            description = "A feature collection.", 
            body = FeatureCollection
        ),
        (
            status = 500, description = "A server error occurred.", 
            body = Exception, example = json!(Exception::new_from_status(500))
        )
    )
)]
pub(crate) async fn search_get(
    State(state): State<AppState>,
    Qs(params): Qs<SearchParams>,
    RemoteUrl(url): RemoteUrl,
) -> Result<(HeaderMap, Json<FeatureCollection>)> {
    search(params, url, state).await
}

/// Search STAC items with full-featured filtering.
///
/// Retrieve Items matching filters. Intended as the standard, full-featured
/// query API.
///
/// This method is optional to implement, but recommended.
///
/// If this endpoint is implemented on a server, it is required to add a
/// link referring to this endpoint with `rel` set to `search` and `method`
/// set to `POST` to the `links` array in `GET /`.
#[utoipa::path(post, path = "/search", tag = "Item Search", 
    request_body = SearchBody,
    responses(
        (
            status = 200,
            description = "A feature collection.", 
            body = FeatureCollection
        ),
        (
            status = 500, description = "A server error occurred.", 
            body = Exception, example = json!(Exception::new_from_status(500))
        )
    )
)]
pub(crate) async fn search_post(
    State(state): State<AppState>,
    RemoteUrl(url): RemoteUrl,
    Json(params): Json<SearchBody>,
) -> Result<(HeaderMap, Json<FeatureCollection>)> {
    search(params.into(), url, state).await
}

pub(crate) async fn search(
    mut params: SearchParams,
    mut url: Url,
    state: AppState,
) -> Result<(HeaderMap, Json<FeatureCollection>)> {
    tracing::debug!("{:#?}", params);

    // Limit
    if let Some(limit) = params.limit {
        if !(1..10001).contains(&limit) {
            return Err(Error::ApiException(
                (
                    StatusCode::BAD_REQUEST,
                    "query parameter `limit` not in range 1 to 10000".to_string(),
                )
                    .into(),
            ));
        }
    } else {
        // default
        params.limit = Some(100);
    }

    // Bbox
    if let Some(bbox) = params.bbox.as_ref() {
        match bbox {
            Bbox::Bbox2D(bbox) => {
                if bbox[0] > bbox[2] || bbox[1] > bbox[3] {
                    return Err(Error::ApiException(
                        (
                            StatusCode::BAD_REQUEST,
                            "query parameter `bbox` not valid".to_string(),
                        )
                            .into(),
                    ));
                }
            }
            Bbox::Bbox3D(bbox) => {
                if bbox[0] > bbox[3] || bbox[1] > bbox[4] || bbox[2] > bbox[5] {
                    return Err(Error::ApiException(
                        (
                            StatusCode::BAD_REQUEST,
                            "query parameter `bbox` not valid".to_string(),
                        )
                            .into(),
                    ));
                }
            }
        }
    }

    let mut fc = state.drivers.stac.search(&params).await?;

    fc.links.insert_or_update(&[
        Link::new(&url, SELF).mediatype(GEO_JSON),
        Link::new(url.join("../..")?, ROOT).mediatype(JSON),
    ]);

    // pagination
    if let Some(limit) = params.limit {
        if params.offset.is_none() {
            params.offset = Some(0);
        }

        if let Some(offset) = params.offset {
            if offset != 0 && offset >= limit {
                params.offset = Some(offset - limit);
                url.set_query(serde_qs::to_string(&params).ok().as_deref());
                let previous = Link::new(&url, PREV).mediatype(GEO_JSON);
                fc.links.insert_or_update(&[previous]);
            }

            if let Some(number_matched) = fc.number_matched
                && number_matched > offset + limit
            {
                params.offset = Some(offset + limit);
                url.set_query(serde_qs::to_string(&params).ok().as_deref());
                let next = Link::new(&url, NEXT).mediatype(GEO_JSON);
                fc.links.insert_or_update(&[next]);
            }
        }
    }

    for feature in fc.features.iter_mut() {
        let collection = feature.collection.as_ref().unwrap();
        feature.links.insert_or_update(&[
            Link::new(
                url.join(&format!(
                    "collections/{}/items/{}",
                    collection,
                    feature.id.as_ref().unwrap()
                ))?,
                SELF,
            )
            .mediatype(GEO_JSON),
            Link::new(url.join(".")?, ROOT).mediatype(JSON),
            Link::new(url.join(&format!("collections/{collection}"))?, COLLECTION).mediatype(JSON),
        ])
    }

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, GEO_JSON.parse().unwrap());

    Ok((headers, Json(fc)))
}

pub(crate) fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(search_get, search_post))
}
