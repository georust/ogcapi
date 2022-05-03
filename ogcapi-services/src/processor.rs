use axum::response::{IntoResponse, Response};

use ogcapi_types::processes::{Execute, InlineOrRefData, Input, InputValueNoObject, Process};

/// Trait for defining and executing a [Process]
pub trait Processor: Send + Sync {
    /// Returns the process id (must be unique)
    fn id(&self) -> String;

    /// Returns the Process description
    fn process(&self) -> Process;

    /// Executes the Process and returns a response
    fn execute(&self, execute: &Execute) -> Response;
}

/// Example Processor
///
/// ```bash
/// curl http://localhost:8484/processes/greet/execution \
///      -H 'Content-Type: application/json' \
///      -d '{"inputs": { "name": "World" } }'
/// ```
pub(crate) struct Greeter;

impl Processor for Greeter {
    fn id(&self) -> String {
        "greet".to_string()
    }
    fn process(&self) -> Process {
        Process::new(
            self.id(),
            "0.1.0",
            &serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" }
                }
            }),
            &serde_json::json!({ "type": "string" }),
        )
    }

    fn execute(&self, execute: &Execute) -> Response {
        let input = execute.inputs.get("name").unwrap();
        match input {
            Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                InputValueNoObject::String(name),
            )) => format!("Hello, {}!", name).into_response(),
            _ => todo!(),
        }
    }
}
