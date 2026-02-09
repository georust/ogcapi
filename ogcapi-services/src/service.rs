use anyhow::Context;
use axum::{
    Router,
    body::Body,
    http::{
        Response, StatusCode,
        header::{AUTHORIZATION, CONTENT_TYPE, COOKIE, PROXY_AUTHORIZATION, SET_COOKIE},
    },
    response::IntoResponse,
};
use ogcapi_types::common::Exception;
use std::{any::Any, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    ServiceBuilderExt,
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
    request_id::MakeRequestUuid,
    sensitive_headers::SetSensitiveRequestHeadersLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use utoipa::{OpenApi as _, openapi::OpenApi};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::{ApiDoc, AppState, Config, ConfigParser, Error, routes, state::Drivers};

/// OGC API Services
pub struct Service {
    pub(crate) state: AppState,
    pub(crate) router: OpenApiRouter<AppState>,
    listener: TcpListener,
    apply_middleware: bool,
    custom_openapi_doc: OpenApi,
}

impl Service {
    pub async fn try_new() -> Result<Self, anyhow::Error> {
        // config
        let config = Config::parse();

        // drivers
        let drivers = Drivers::try_new_from_env().await?;

        // state
        let state = AppState::new(drivers).await;

        Service::try_new_with(&config, state).await
    }

    pub async fn try_new_with(config: &Config, state: AppState) -> Result<Self, anyhow::Error> {
        // router
        let router = OpenApiRouter::<AppState>::with_openapi(ApiDoc::openapi());

        let router = router.merge(routes::common::router());

        // add a fallback service for handling routes to unknown paths
        let router = router.fallback(handler_404);

        // listen
        let listener = TcpListener::bind((config.host.as_str(), config.port)).await?;

        Ok(Service {
            state,
            router,
            listener,
            apply_middleware: true,
            custom_openapi_doc: OpenApi::default(),
        })
    }

    pub fn with_collections_api(mut self) -> Self {
        self.router = self.router.merge(routes::collections::router(&self.state));
        self
    }

    #[cfg(feature = "features")]
    pub fn with_features_api(mut self) -> Self {
        self.router = self.router.merge(routes::features::router(&self.state));
        self
    }

    #[cfg(feature = "edr")]
    pub fn with_edr_api(mut self) -> Self {
        self.router = self.router.merge(routes::edr::router(&self.state));
        self
    }

    #[cfg(feature = "styles")]
    pub fn with_styles_api(mut self) -> Self {
        self.router = self.router.merge(routes::styles::router(&self.state));
        self
    }

    #[cfg(feature = "stac")]
    pub fn with_stac_api(mut self) -> Self {
        self.router = self.router.merge(routes::stac::router());
        self
    }

    #[cfg(feature = "tiles")]
    pub fn with_tiles_api(mut self) -> Self {
        self.router = self.router.merge(routes::tiles::router(&self.state));
        self
    }

    #[cfg(feature = "processes")]
    pub fn with_processes_api(mut self) -> Self {
        self.router = self.router.merge(routes::processes::router(&self.state));
        self
    }

    /// Add all available APIs to the service
    pub fn with_all_apis(mut self) -> Self {
        self = self.with_collections_api();

        #[cfg(feature = "features")]
        {
            self = self.with_features_api();
        }

        #[cfg(feature = "edr")]
        {
            self = self.with_edr_api();
        }

        #[cfg(feature = "styles")]
        {
            self = self.with_styles_api();
        }

        #[cfg(feature = "stac")]
        {
            self = self.with_stac_api();
        }

        #[cfg(feature = "tiles")]
        {
            self = self.with_tiles_api();
        }

        #[cfg(feature = "processes")]
        {
            self = self.with_processes_api();
        }

        self
    }

    /// Customize the router by providing a function that takes the current router and returns a modified router.
    /// This can be used to add custom routes or middleware to the service.
    pub fn with_custom_router(
        mut self,
        router_fn: impl FnOnce(OpenApiRouter<AppState>) -> OpenApiRouter<AppState>,
    ) -> Self {
        self.router = router_fn(self.router);
        self
    }

    /// Set additional [`OpenApi`] document for the service, which is empty by default.
    /// This can be used to add custom paths or components to the [`OpenApi`] document.
    /// The provided [`OpenApi`] document will be merged with the auto-generated one from the [`Service`].
    ///
    /// Note: The provided [`OpenApi`] document should not contain any paths or components that are already defined by the service, as this may lead to conflicts.
    pub fn with_custom_openapi_doc(mut self, openapi_doc: OpenApi) -> Self {
        self.custom_openapi_doc = openapi_doc;
        self
    }

    /// Serve application
    pub async fn serve(self) -> Result<(), anyhow::Error> {
        // api documentation
        let (router, api) = self.router.split_for_parts();

        let openapi = Arc::new(api.merge_from(self.custom_openapi_doc));

        let router =
            router.merge(SwaggerUi::new("/swagger").url("/api_v3.1", openapi.as_ref().clone()));

        // add state
        let router =
            Service::apply_middleware(router, self.apply_middleware).with_state(self.state);

        // serve
        tracing::info!(
            "listening on http://{}",
            self.listener.local_addr()?.to_string()
        );

        axum::serve::serve(self.listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .context("failed to serve application")?;

        Ok(())
    }

    fn apply_middleware(router: Router<AppState>, apply_middleware: bool) -> Router<AppState> {
        if !apply_middleware {
            return router;
        }

        let middleware_stack = ServiceBuilder::new()
            .set_x_request_id(MakeRequestUuid)
            .layer(SetSensitiveRequestHeadersLayer::new([
                AUTHORIZATION,
                PROXY_AUTHORIZATION,
                COOKIE,
                SET_COOKIE,
            ]))
            .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new()))
            .layer(CompressionLayer::new())
            .layer(CorsLayer::permissive())
            .layer(CatchPanicLayer::custom(handle_panic))
            .propagate_x_request_id();

        router.layer(middleware_stack)
    }

    /// Do not apply middleware layers
    pub fn without_middleware(mut self) -> Self {
        self.apply_middleware = false;
        self
    }

    // helper function to get randomized port
    #[doc(hidden)]
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.listener.local_addr()
    }
}

/// Custom 404 handler
async fn handler_404() -> impl IntoResponse {
    Error::NotFound
}

/// Custom panic handler
fn handle_panic(err: Box<dyn Any + Send + 'static>) -> Response<Body> {
    let details = if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "Unknown panic message".to_string()
    };

    let body =
        Exception::new_from_status(StatusCode::INTERNAL_SERVER_ERROR.as_u16()).detail(details);

    let body = serde_json::to_string(&body).unwrap();

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap()
}

/// Handle shutdown signals
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::debug!("signal received, starting graceful shutdown");
}
