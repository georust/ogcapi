use anyhow::Result;
use dyn_clone::DynClone;
use ogcapi_types::processes::{Execute, ExecuteResults, Process, ProcessAuthRequirement};

/// Trait for defining and executing a [Process]
#[async_trait::async_trait]
pub trait Processor: Send + Sync + DynClone {
    type User;

    /// Returns the process id (must be unique)
    fn id(&self) -> &'static str;

    /// Returns the process version
    fn version(&self) -> &'static str;

    /// Returns the Process description
    fn process(&self) -> Result<Process>;

    /// Executes the Process and returns [Results]
    async fn execute(&self, execute: Execute, user: &Self::User) -> Result<ExecuteResults>;

    /// Whether the process requires authentication
    fn requires_auth(&self) -> ProcessAuthRequirement {
        ProcessAuthRequirement::NoAuth
    }
}

dyn_clone::clone_trait_object!(<User> Processor<User = User>);
