pub(crate) mod api;
pub(crate) mod collections;
#[cfg(feature = "edr")]
pub(crate) mod edr;
#[cfg(feature = "features")]
pub(crate) mod features;
#[cfg(feature = "processes")]
pub(crate) mod processes;
#[cfg(feature = "stac")]
pub(crate) mod stac;
#[cfg(feature = "styles")]
pub(crate) mod styles;
#[cfg(feature = "tiles")]
pub(crate) mod tiles;

use axum::{Json, extract::State};

#[cfg(feature = "stac")]
use ogcapi_types::common::link_rel::SEARCH;
use ogcapi_types::common::{
    Conformance, LandingPage, Link, Linked,
    link_rel::{CONFORMANCE, ROOT, SELF, SERVICE_DESC, SERVICE_DOC},
    media_type::{HTML, JSON, OPEN_API_JSON},
};

use crate::{AppState, Result, extractors::RemoteUrl};

pub(crate) async fn root(
    RemoteUrl(url): RemoteUrl,
    State(state): State<AppState>,
) -> Result<Json<LandingPage>> {
    let mut root = state.root.read().unwrap().to_owned();

    root.links.insert_or_update(&[
        Link::new(format!("{}/", url.as_str().trim_end_matches('/')), SELF).mediatype(JSON),
        Link::new(".", ROOT).mediatype(JSON),
        Link::new("api", SERVICE_DESC)
            .title("The Open API definition")
            .mediatype(OPEN_API_JSON),
        Link::new("swagger", SERVICE_DOC)
            .title("The Open API definition (Swagger UI)")
            .mediatype(HTML),
        // Link::new("redoc", SERVICE_DOC)
        //     .title("The Open API definition (Redoc")
        //     .mediatype(HTML),
        Link::new("conformance", CONFORMANCE)
            .title("Conformance classes implemented by this API")
            .mediatype(JSON),
        #[cfg(feature = "stac")]
        Link::new("search", SEARCH)
            .title("URI for the STAC API - Item Search endpoint")
            .mediatype(JSON),
    ]);
    root.links.resolve_relative_links();

    #[cfg(feature = "stac")]
    let root = root.conforms_to(&state.conformance.read().unwrap().conforms_to[..]);

    Ok(Json(root))
}

pub(crate) async fn conformance(State(state): State<AppState>) -> Json<Conformance> {
    Json(state.conformance.read().unwrap().to_owned())
}
