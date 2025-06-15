use axum::{Json, extract::State};

#[cfg(feature = "stac")]
use ogcapi_types::common::link_rel::SEARCH;
use ogcapi_types::common::{
    Conformance, Exception, LandingPage, Link, Linked,
    link_rel::{CONFORMANCE, ROOT, SELF, SERVICE_DESC, SERVICE_DOC},
    media_type::{HTML, JSON, OPEN_API_JSON},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{AppState, Result, extractors::RemoteUrl};

/// Landing page
///
/// The landing page provides links to the API definition and the
/// conformance statements for this API.
#[utoipa::path(
    get, path = "/", tag = "Capabilities", 
    responses(
        (
            status = 200,
            description = "The landing page provides links to the API \
            definition (link relations `service-desc` and `service-doc`), and \
            the Conformance declaration (path `/conformance`, link relation \
            `conformance`).", 
            body = LandingPage
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
pub async fn root(
    State(state): State<AppState>,
    RemoteUrl(url): RemoteUrl,
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

/// API conformance definition
///
/// A list of all conformance classes specified in a standard that
/// the server conforms to.
#[utoipa::path(
    get, path = "/conformance", tag = "Capabilities", 
    responses(
        (
            status = 200,
            description = "The URIs of all conformance classes supported by the server.\
                \n\n To support \"generic\" clients that want to access multiple \
                OGC API Features implementations - and not \"just\" a specific \
                API / server, the server declares the conformance classes it \
                implements and conforms to", 
            body = Conformance
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
pub(crate) async fn conformance(State(state): State<AppState>) -> Json<Conformance> {
    Json(state.conformance.read().unwrap().to_owned())
}

pub(crate) fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(root))
        .routes(routes!(conformance))
}
