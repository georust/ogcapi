pub(crate) mod api;
pub(crate) mod collections;
#[cfg(feature = "edr")]
pub(crate) mod edr;
#[cfg(feature = "features")]
pub(crate) mod features;
#[cfg(feature = "processes")]
pub(crate) mod processes;
#[cfg(feature = "styles")]
pub(crate) mod styles;
#[cfg(feature = "tiles")]
pub(crate) mod tiles;

use std::sync::Arc;

use axum::{extract::Extension, Json};

use ogcapi_types::common::{
    link_rel::{CONFORMANCE, SELF, SERVICE_DESC},
    media_type::{JSON, OPEN_API_JSON},
    Conformance, LandingPage, Link, Linked,
};

use crate::{extractors::RemoteUrl, Result, State};

pub(crate) async fn root(
    RemoteUrl(url): RemoteUrl,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<LandingPage>> {
    let mut root = state.root.read().unwrap().to_owned();

    root.links.insert_or_update(&[
        Link::new(url, SELF).title("This document").mediatype(JSON),
        Link::new("api", SERVICE_DESC)
            .title("The Open API definition")
            .mediatype(OPEN_API_JSON),
        Link::new("conformance", CONFORMANCE)
            .title("OGC conformance classes implemented by this API")
            .mediatype(JSON),
    ]);
    root.links.resolve_relative_links();

    Ok(Json(root))
}

pub(crate) async fn conformance(Extension(state): Extension<Arc<State>>) -> Json<Conformance> {
    Json(state.conformance.read().unwrap().to_owned())
}
