use std::sync::Arc;

use axum::{http::HeaderMap, Extension, Json};
use hyper::header::CONTENT_TYPE;
use ogcapi_drivers::StacSeach;
use ogcapi_types::{
    common::{
        link_rel::{COLLECTION, NEXT, PREV, ROOT, SELF},
        media_type::{GEO_JSON, JSON},
        Link, Linked,
    },
    features::FeatureCollection,
    stac::SearchParams,
};

use crate::{
    extractors::{Qs, RemoteUrl},
    Result, State,
};

pub(crate) async fn search(
    Qs(mut query): Qs<SearchParams>,
    RemoteUrl(mut url): RemoteUrl,
    Extension(state): Extension<Arc<State>>,
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

    let mut fc = state.db.search(&query).await?;

    fc.links.insert_or_update(&[
        Link::new(&url, SELF).mediatype(GEO_JSON),
        Link::new(&url.join("../..")?, ROOT).mediatype(JSON),
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

            if let Some(number_matched) = fc.number_matched {
                if number_matched > (offset + limit) as u64 {
                    query.offset = Some(offset + limit);
                    url.set_query(serde_qs::to_string(&query).ok().as_deref());
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
                &url.join(&format!(
                    "collections/{}/items/{}",
                    collection,
                    feature.id.as_ref().unwrap()
                ))?,
                SELF,
            )
            .mediatype(GEO_JSON),
            Link::new(&url.join(".")?, ROOT).mediatype(JSON),
            Link::new(
                &url.join(&format!("collections/{}", collection))?,
                COLLECTION,
            )
            .mediatype(JSON),
        ])
    }

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, GEO_JSON.parse().unwrap());

    Ok((headers, Json(fc)))
}
