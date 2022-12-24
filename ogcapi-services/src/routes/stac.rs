use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use hyper::header::CONTENT_TYPE;
use ogcapi_drivers::StacSeach;
use ogcapi_types::{
    common::{
        link_rel::{COLLECTION, NEXT, PREV, ROOT, SELF},
        media_type::{GEO_JSON, JSON},
        Bbox, Link, Linked,
    },
    features::FeatureCollection,
    stac::{SearchBody, SearchParams},
};
use url::Url;

use crate::{
    extractors::{Qs, RemoteUrl},
    AppState, Error, Result,
};

pub(crate) async fn search_get(
    State(state): State<AppState>,
    Qs(params): Qs<SearchParams>,
    RemoteUrl(url): RemoteUrl,
) -> Result<(HeaderMap, Json<FeatureCollection>)> {
    search(params, url, state).await
}

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
            return Err(Error::Exception(
                StatusCode::BAD_REQUEST,
                "query parameter `limit` not in range 1 to 10000".to_string(),
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
                    return Err(Error::Exception(
                        StatusCode::BAD_REQUEST,
                        "query parameter `bbox` not valid".to_string(),
                    ));
                }
            }
            Bbox::Bbox3D(bbox) => {
                if bbox[0] > bbox[3] || bbox[1] > bbox[4] || bbox[2] > bbox[5] {
                    return Err(Error::Exception(
                        StatusCode::BAD_REQUEST,
                        "query parameter `bbox` not valid".to_string(),
                    ));
                }
            }
        }
    }

    let mut fc = state.db.search(&params).await?;

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

            if let Some(number_matched) = fc.number_matched {
                if number_matched > offset + limit {
                    params.offset = Some(offset + limit);
                    url.set_query(serde_qs::to_string(&params).ok().as_deref());
                    let next = Link::new(&url, NEXT).mediatype(GEO_JSON);
                    fc.links.insert_or_update(&[next]);
                }
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
            Link::new(
                url.join(&format!("collections/{}", collection))?,
                COLLECTION,
            )
            .mediatype(JSON),
        ])
    }

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, GEO_JSON.parse().unwrap());

    Ok((headers, Json(fc)))
}
