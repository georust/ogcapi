mod config;
mod error;
mod extractors;
mod openapi;
#[cfg(feature = "processes")]
mod processor;
mod routes;
mod service;
mod state;
pub mod telemetry;

pub use config::Config;
pub use error::Error;
pub use openapi::OpenAPI;
pub use service::Service;
pub use state::State;

#[cfg(feature = "processes")]
pub use processor::{Greeter, Processor};

#[doc(hidden)]
pub use clap::Parser as ConfigParser;

pub type Result<T, E = Error> = std::result::Result<T, E>;
