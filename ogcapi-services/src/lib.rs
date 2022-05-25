mod config;
mod error;
mod extractors;
#[cfg(feature = "processes")]
mod processor;
mod routes;
mod state;

pub use config::{parse_config, Config};
pub use error::Error;
pub use state::State;

#[cfg(feature = "processes")]
pub use processor::{Greeter, Processor};

use std::sync::Arc;

use axum::{extract::Extension, routing::get, Router};
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub static OPENAPI: &[u8; 29696] = include_bytes!("../openapi.yaml");

pub async fn app(state: State) -> Router {
    // routes
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

    // middleware stack
    router.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new())
            .layer(CorsLayer::permissive())
            .layer(Extension(Arc::new(state))),
    )
}

/// Handle shutdown signals
pub async fn shutdown_signal() {
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
