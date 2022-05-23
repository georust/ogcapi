use axum::{
    async_trait,
    response::{IntoResponse, Response},
};

// use ogcapi_drivers::s3::{ByteStream, S3};
use ogcapi_types::processes::{Execute, Process};
use schemars::{schema_for, JsonSchema};
use serde::Deserialize;

use crate::{Result, State};

#[async_trait]
/// Trait for defining and executing a [Process]
pub trait Processor: Send + Sync {
    /// Returns the process id (must be unique)
    fn id(&self) -> String;

    /// Returns the Process description
    fn process(&self) -> Process;

    /// Executes the Process and returns a response
    async fn execute(&self, execute: Execute, state: &State) -> Result<Response>;
}

/// Example Processor
///
/// ```bash
/// curl http://localhost:8484/processes/greet/execution \
///         -u 'user:password'
///         -H 'Content-Type: application/json' \
///         -d '{"inputs": { "name": "World" } }'
/// ```
pub(crate) struct Greeter;

/// Inputs for the `greet` process
#[derive(Deserialize, Debug, JsonSchema)]
struct GreeterInputs {
    /// Name to be greeted
    name: String,
}

/// Outputs for the `greet` process
#[derive(JsonSchema)]
struct GreeterOutputs(String);

#[async_trait]
impl Processor for Greeter {
    fn id(&self) -> String {
        "greet".to_string()
    }
    fn process(&self) -> Process {
        Process::new(
            self.id(),
            "0.1.0",
            &serde_json::to_value(&schema_for!(GreeterInputs).schema).unwrap(),
            &serde_json::to_value(&schema_for!(GreeterOutputs).schema).unwrap(),
        )
    }

    async fn execute(&self, execute: Execute, _state: &State) -> Result<Response> {
        let value = serde_json::to_value(execute.inputs).unwrap();
        let inputs: GreeterInputs = serde_json::from_value(value).unwrap();
        Ok(format!("Hello, {}!", inputs.name).into_response())
    }
}
