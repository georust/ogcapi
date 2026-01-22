use axum::{Json, extract::State};
use hyper::{HeaderMap, header::CONTENT_TYPE};
use utoipa_axum::{router::OpenApiRouter, routes};

#[cfg(feature = "stac")]
use ogcapi_types::common::link_rel::SEARCH;
use ogcapi_types::common::{
    Conformance, Exception, LandingPage, Link, Linked,
    link_rel::{CONFORMANCE, ROOT, SELF, SERVICE_DESC, SERVICE_DOC},
    media_type::{HTML, JSON, OPEN_API_JSON},
};

use crate::{
    Result,
    extractors::RemoteUrl,
    openapi::{OPENAPI, OpenAPI},
    routes2,
    state::OgcApiState,
};

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
pub async fn root<S: OgcApiState>(
    State(state): State<S>,
    RemoteUrl(url): RemoteUrl,
) -> Result<Json<LandingPage>> {
    let mut root = state.root();

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
    let root = root.conforms_to(&state.conformance().conforms_to);

    Ok(Json(root))
}

/// API definition
#[utoipa::path(
    get, path = "/api", tag = "Capabilities", 
    responses(
        (
            status = 200,
            description = "The Open API definition.", 
            body = serde_json::Map<String, Value>,
        ),
        (
            status = 500, description = "A server error occurred.", 
            body = Exception, example = json!(Exception::new_from_status(500))
        )
    )
)]
pub(crate) async fn api(RemoteUrl(url): RemoteUrl) -> (HeaderMap, Json<openapiv3::OpenAPI>) {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, OPEN_API_JSON.parse().unwrap());

    let mut open_api = OpenAPI::from_slice(OPENAPI).0;

    let base_url = url[..url::Position::BeforePath].to_string();

    // replace servers with relative server
    open_api.servers = vec![openapiv3::Server {
        url: base_url,
        description: None,
        variables: None,
        extensions: Default::default(),
    }];

    (headers, Json(open_api))
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
pub(crate) async fn conformance<S: OgcApiState>(State(state): State<S>) -> Json<Conformance> {
    Json(state.conformance())
}

pub(crate) fn router<S: OgcApiState>() -> OpenApiRouter<S> {
    OpenApiRouter::new()
        .routes(routes2!(root))
        .routes(routes!(api))
        .routes(routes2!(conformance))
}
