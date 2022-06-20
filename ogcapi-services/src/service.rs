use std::{
    any::Any,
    net::{SocketAddr, TcpListener},
    sync::Arc,
};

use axum::{
    body::Body,
    handler::Handler,
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE, COOKIE, PROXY_AUTHORIZATION, SET_COOKIE},
        Response, StatusCode,
    },
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
    request_id::MakeRequestUuid,
    sensitive_headers::SetSensitiveRequestHeadersLayer,
    trace::{DefaultMakeSpan, TraceLayer},
    ServiceBuilderExt,
};

use ogcapi_types::common::Exception;

use crate::{routes, Config, ConfigParser, Error, State};

/// OGC API Services
pub struct Service {
    pub state: State,
    pub router: Router,
    listener: TcpListener,
}

impl Service {
    pub async fn new() -> Self {
        // config
        let config = Config::parse();

        // state
        let state = State::new_from(&config).await;

        Service::new_with(&config, state).await
    }

    pub async fn new_with(config: &Config, state: State) -> Self {
        // router
        let router = Router::new()
            .route("/", get(routes::root))
            .route("/api", get(routes::api::api))
            .route("/redoc", get(routes::api::redoc))
            .route("/swagger", get(routes::api::swagger))
            .route("/conformance", get(routes::conformance));

        let router = router.merge(routes::collections::router(&state));

        #[cfg(feature = "features")]
        let router = router.merge(routes::features::router(&state));

        #[cfg(feature = "edr")]
        let router = router.merge(routes::edr::router(&state));

        #[cfg(feature = "styles")]
        let router = router.merge(routes::styles::router(&state));

        #[cfg(feature = "tiles")]
        let router = router.merge(routes::tiles::router(&state));

        #[cfg(feature = "processes")]
        let router = router.merge(routes::processes::router(&state));

        // add a fallback service for handling routes to unknown paths
        let router = router.fallback(handler_404.into_service());

        // middleware stack
        let router = router.layer(
            ServiceBuilder::new()
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
                .propagate_x_request_id(),
        );

        let listener =
            TcpListener::bind((config.host.as_str(), config.port)).expect("create listener");

        Service {
            state,
            router,
            listener,
        }
    }

    /// Serve application
    pub async fn serve(self) {
        // add state
        let router = self.router.layer(Extension(Arc::new(self.state)));

        // serve
        tracing::info!(
            "listening on http://{}",
            self.listener.local_addr().unwrap()
        );

        let builder = axum::Server::from_tcp(self.listener).unwrap();
        builder
            .serve(router.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap()
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
