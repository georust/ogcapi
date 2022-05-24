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

use ogcapi_types::common::{link_rel::SELF, Conformance, LandingPage, Link, Linked};

use crate::{extractors::RemoteUrl, Result, State};

pub(crate) async fn root(
    RemoteUrl(url): RemoteUrl,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<LandingPage>> {
    let mut root = state.root.read().expect("Read root from state").clone();

    root.links
        .iter_mut()
        .find(|l| l.rel == SELF)
        .map(|l| l.href = url.to_string())
        .unwrap_or_else(|| root.links.insert(0, Link::new(url, SELF)));

    root.links.resolve_relative_links();

    Ok(Json(root))
}

pub(crate) async fn conformance(Extension(state): Extension<Arc<State>>) -> Json<Conformance> {
    Json(state.conformance.read().unwrap().to_owned())
}
