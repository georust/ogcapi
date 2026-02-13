use std::{any::Any, collections::HashSet, convert::Infallible, net::SocketAddr, sync::Arc};

use anyhow::Context;
use axum::{
    body::Body,
    extract::Request,
    http::{
        StatusCode,
        header::{AUTHORIZATION, CONTENT_TYPE, COOKIE, PROXY_AUTHORIZATION, SET_COOKIE},
    },
    response::{IntoResponse, Response},
    routing::Route,
};
use tokio::net::TcpListener;
use tower::{ServiceBuilder, util::BoxCloneSyncServiceLayer};
use tower_http::{
    ServiceBuilderExt,
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
    map_response_body::MapResponseBodyLayer,
    request_id::MakeRequestUuid,
    sensitive_headers::SetSensitiveRequestHeadersLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use utoipa::OpenApi as _;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use ogcapi_types::common::Exception;

use crate::{ApiDoc, AppState, Config, ConfigParser, Error, routes, state::Drivers};

/// OGC API Services
pub struct Service {
    pub(crate) state: AppState,
    pub(crate) router: OpenApiRouter<AppState>,
    listener: TcpListener,
    middleware_stack: BoxCloneSyncServiceLayer<Route, Request, Response, Infallible>,
    /// Prevent multiple additions of the same API to the service, which would cause duplicate routes and documentation.
    added_apis: HashSet<ApiType>,
}

impl Service {
    /// Create a new service by reading the configuration from environment variables and command line arguments.
    /// Proceeds to call [`try_new()`](Self::try_new) with the parsed configuration and application state.
    pub async fn try_new_from_env() -> Result<Self, anyhow::Error> {
        // config
        let config = Config::parse();

        // drivers
        let drivers = Drivers::try_new_from_env().await?;

        // state
        let state = AppState::new(drivers).await;

        Service::try_new(&config, state).await
    }

    /// Create a new service with the given configuration and application state.
    ///
    /// This function sets up the router, listener, and middleware stack for the service.
    /// It also adds a fallback route for handling requests to unknown paths.
    /// The service is not started yet, you need to call [`serve()`](Self::serve) to start the server.
    ///
    /// Note, this function only adds the common routes to the router, you need to call the respective API functions (e.g. [`collections_api()`](Self::collections_api) or [`all_apis()`](Self::all_apis)) to add the specific API routes to the router.
    pub async fn try_new(config: &Config, state: AppState) -> Result<Self, anyhow::Error> {
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
            middleware_stack: default_middleware_stack(),
            added_apis: HashSet::new(),
        })
    }

    pub fn collections_api(mut self) -> Self {
        if self.added_apis.insert(ApiType::Collections) {
            self.router = self.router.merge(routes::collections::router(&self.state));
        }
        self
    }

    #[cfg(feature = "features")]
    pub fn features_api(mut self) -> Self {
        if self.added_apis.insert(ApiType::Features) {
            self.router = self.router.merge(routes::features::router(&self.state));
        }
        self
    }

    #[cfg(feature = "edr")]
    pub fn edr_api(mut self) -> Self {
        if self.added_apis.insert(ApiType::Edr) {
            self.router = self.router.merge(routes::edr::router(&self.state));
        }
        self
    }

    #[cfg(feature = "styles")]
    pub fn styles_api(mut self) -> Self {
        if self.added_apis.insert(ApiType::Styles) {
            self.router = self.router.merge(routes::styles::router(&self.state));
        }
        self
    }

    #[cfg(feature = "stac")]
    pub fn stac_api(mut self) -> Self {
        if self.added_apis.insert(ApiType::Stac) {
            self.router = self.router.merge(routes::stac::router());
        }
        self
    }

    #[cfg(feature = "tiles")]
    pub fn tiles_api(mut self) -> Self {
        if self.added_apis.insert(ApiType::Tiles) {
            self.router = self.router.merge(routes::tiles::router(&self.state));
        }
        self
    }

    #[cfg(feature = "processes")]
    pub fn processes_api(mut self) -> Self {
        if self.added_apis.insert(ApiType::Processes) {
            self.router = self.router.merge(routes::processes::router(&self.state));
        }
        self
    }

    /// Add all available APIs to the service
    pub fn all_apis(mut self) -> Self {
        self = self.collections_api();

        #[cfg(feature = "features")]
        {
            self = self.features_api();
        }

        #[cfg(feature = "edr")]
        {
            self = self.edr_api();
        }

        #[cfg(feature = "styles")]
        {
            self = self.styles_api();
        }

        #[cfg(feature = "stac")]
        {
            self = self.stac_api();
        }

        #[cfg(feature = "tiles")]
        {
            self = self.tiles_api();
        }

        #[cfg(feature = "processes")]
        {
            self = self.processes_api();
        }

        self
    }

    /// Allows modifying the router by providing a function that takes the current router and returns a modified router.
    /// This can be used to add custom routes or middleware to the service.
    pub fn get_router_mut(&mut self) -> &mut OpenApiRouter<AppState> {
        &mut self.router
    }

    /// Get a mutable reference to the middleware stack, allowing for modification of the middleware layers.
    pub fn get_middleware_stack_mut(
        &mut self,
    ) -> &mut BoxCloneSyncServiceLayer<Route, Request, Response, Infallible> {
        &mut self.middleware_stack
    }

    /// Serve application
    pub async fn serve(self) -> Result<(), anyhow::Error> {
        // api documentation
        let (router, api) = self.router.split_for_parts();

        let openapi = Arc::new(api);

        let router =
            router.merge(SwaggerUi::new("/swagger").url("/api_v3.1", openapi.as_ref().clone()));

        // add state
        let router = router.layer(self.middleware_stack).with_state(self.state);

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

    // helper function to get randomized port
    #[doc(hidden)]
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.listener.local_addr()
    }
}

fn default_middleware_stack() -> BoxCloneSyncServiceLayer<Route, Request, Response, Infallible> {
    let inner = ServiceBuilder::new()
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
        .propagate_x_request_id()
        .into_inner();

    let inner = (MapResponseBodyLayer::new(Body::new), inner); // erase complex type after compression layer

    BoxCloneSyncServiceLayer::new(inner)
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

#[derive(Debug, Hash, PartialEq, Eq)]
enum ApiType {
    Collections,
    #[cfg(feature = "features")]
    Features,
    #[cfg(feature = "edr")]
    Edr,
    #[cfg(feature = "styles")]
    Styles,
    #[cfg(feature = "stac")]
    Stac,
    #[cfg(feature = "tiles")]
    Tiles,
    #[cfg(feature = "processes")]
    Processes,
}
