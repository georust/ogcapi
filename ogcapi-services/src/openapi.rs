use utoipa::OpenApi;

/// Open API documentation
#[derive(OpenApi)]
#[openapi(
    // paths(openapi),
    info(
        contact(
            name = "GeoRust `ogcapi` project on GitHub",
            url = "https://github.com/georust/ogcapi",
        )
    )
)]
pub struct ApiDoc;
