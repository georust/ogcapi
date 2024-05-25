use axum::response::Response;
use dyn_clone::DynClone;
use url::Url;

use ogcapi_types::processes::{Execute, Process};

use crate::{AppState, Result};

/// Trait for defining and executing a [Process]
#[axum::async_trait]
pub trait Processor: Send + Sync + DynClone {
    /// Returns the process id (must be unique)
    fn id(&self) -> String;

    /// Returns the Process description
    fn process(&self) -> Process;

    /// Executes the Process and returns a response
    async fn execute(&self, execute: Execute, state: &AppState, url: &Url) -> Result<Response>;
}

dyn_clone::clone_trait_object!(Processor);

#[cfg(feature = "greeter")]
mod greeter;
#[cfg(feature = "greeter")]
pub use greeter::Greeter;
