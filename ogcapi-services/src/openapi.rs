use utoipa::OpenApi;

/// TODO: remove once Open API 3.1 is supported
#[cfg(all(feature = "features", not(feature = "edr")))]
pub(crate) static OPENAPI: &[u8; 29696] = include_bytes!("../assets/openapi/openapi.yaml");

/// TODO: remove once Open API 3.1 is supported
#[cfg(feature = "edr")]
pub(crate) static OPENAPI: &[u8; 764046] = include_bytes!("../assets/openapi/openapi-processes.yaml");

#[derive(Default, Clone)]
pub(crate) struct OpenAPI(pub(crate) openapiv3::OpenAPI);

impl OpenAPI {
    pub(crate) fn from_slice(api: &[u8]) -> Self {
        let openapi: openapiv3::OpenAPI = serde_yaml::from_slice(api).unwrap();
        OpenAPI(openapi)
    }
}

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
