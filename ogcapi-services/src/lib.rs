mod config;
mod error;
mod extractors;
mod openapi;
#[cfg(feature = "processes")]
pub mod processes;
mod routes;
mod service;
mod state;
pub mod telemetry;

pub use config::Config;
pub use error::Error;
pub use openapi::OpenAPI;
pub use service::Service;
pub use state::AppState;

#[cfg(feature = "processes")]
pub use processes::Processor;

#[doc(hidden)]
pub use clap::Parser as ConfigParser;

pub type Result<T, E = Error> = std::result::Result<T, E>;
