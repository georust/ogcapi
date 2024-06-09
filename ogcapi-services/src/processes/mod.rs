use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use dyn_clone::DynClone;
use url::Url;

use ogcapi_types::processes::{Execute, Process, Results, StatusInfo};

use crate::{AppState, Result};

/// Trait for defining and executing a [Process]
#[axum::async_trait]
pub trait Processor: Send + Sync + DynClone {
    /// Returns the process id (must be unique)
    fn id(&self) -> String;

    /// Returns the Process description
    fn process(&self) -> Process;

    /// Executes the Process and returns [Results]
    async fn execute(&self, execute: Execute, state: &AppState, url: &Url)
        -> Result<ProcessOutput>;
}

dyn_clone::clone_trait_object!(Processor);

pub enum ProcessOutput {
    Requested(Response),
    Results(Results),
    Empty,
    StatusInfo(StatusInfo),
}

impl IntoResponse for ProcessOutput {
    fn into_response(self) -> Response {
        match self {
            ProcessOutput::Requested(request) => request,
            ProcessOutput::Results(results) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&results).unwrap()))
                .unwrap(),
            ProcessOutput::Empty => Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(Body::empty())
                .unwrap(),
            ProcessOutput::StatusInfo(status_info) => Response::builder()
                .status(StatusCode::CREATED)
                .header("Content-Type", "application/json")
                .header("Location", &format!("../../jobs/{}", status_info.job_id))
                .body(Body::from(serde_json::to_vec(&status_info).unwrap()))
                .unwrap(),
        }
    }
}

#[cfg(feature = "greeter")]
mod greeter;
#[cfg(feature = "greeter")]
pub use greeter::Greeter;
