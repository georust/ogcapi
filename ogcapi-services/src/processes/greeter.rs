use axum::response::IntoResponse;
use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use url::Url;

use ogcapi_types::processes::{Execute, Process};

use crate::{AppState, Processor, Result};

use super::ProcessOutput;

/// Greeter `Processor`
///
/// Example processor that takes a name as input and returns a greeting.
///
/// # Usage
///
/// ```bash
/// curl http://localhost:8484/processes/greet/execution \
///         -H 'Content-Type: application/json' \
///         -d '{ "inputs": { "name": "World" } }'
/// ```
#[derive(Clone)]
pub struct Greeter;

/// Inputs for the `greet` process
#[derive(Deserialize, Debug, JsonSchema)]
struct GreeterInputs {
    /// Name to be greeted
    name: String,
}

/// Outputs for the `greet` process
#[allow(dead_code)]
#[derive(JsonSchema)]
struct GreeterOutputs(String);

#[axum::async_trait]
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

    async fn execute(
        &self,
        execute: Execute,
        _state: &AppState,
        _url: &Url,
    ) -> Result<ProcessOutput> {
        let value = serde_json::to_value(execute.inputs).unwrap();
        let inputs: GreeterInputs = serde_json::from_value(value).unwrap();
        Ok(ProcessOutput::Requested(
            format!("Hello, {}!\n", inputs.name).into_response(),
        ))
    }
}
