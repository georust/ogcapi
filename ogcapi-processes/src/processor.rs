use anyhow::Result;
use dyn_clone::DynClone;

use ogcapi_types::processes::{Execute, Process, Results, StatusInfo};

/// Trait for defining and executing a [Process]
#[async_trait::async_trait]
pub trait Processor: Send + Sync + DynClone {
    /// Returns the process id (must be unique)
    fn id(&self) -> &'static str;

    /// Returns the process version
    fn version(&self) -> &'static str;

    /// Returns the Process description
    fn process(&self) -> Result<Process>;

    /// Executes the Process and returns [Results]
    async fn execute(
        &self,
        execute: Execute,
        // state: &AppState,
        // url: &Url,
    ) -> Result<ProcessResponseBody>;
}

dyn_clone::clone_trait_object!(Processor);

pub enum ProcessResponseBody {
    Requested(Vec<u8>),
    Results(Results),
    Empty,
    StatusInfo(StatusInfo),
}
