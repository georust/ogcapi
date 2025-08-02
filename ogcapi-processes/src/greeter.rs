use std::collections::HashMap;

use anyhow::Result;
use schemars::{JsonSchema, schema_for};
use serde::Deserialize;

use ogcapi_types::{
    common::Exception,
    processes::{
        Execute, Format, InlineOrRefData, Input, InputValueNoObject, Output, Process,
        TransmissionMode,
    },
};

use crate::{ProcessResponseBody, Processor};

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
pub struct GreeterInputs {
    /// Name to be greeted
    pub name: String,
}

impl GreeterInputs {
    pub fn execute_input(&self) -> HashMap<String, Input> {
        HashMap::from([(
            "name".to_string(),
            Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                InputValueNoObject::String(self.name.to_owned()),
            )),
        )])
    }
}

/// Outputs for the `greet` process
#[derive(Clone, Debug, JsonSchema)]
pub struct GreeterOutputs {
    pub greeting: String,
}

impl GreeterOutputs {
    pub fn execute_output() -> HashMap<String, Output> {
        HashMap::from([(
            "greeting".to_string(),
            Output {
                format: Some(Format {
                    media_type: Some("text/plain".to_string()),
                    encoding: Some("utf8".to_string()),
                    schema: None,
                }),
                transmission_mode: TransmissionMode::Value,
            },
        )])
    }
}

impl TryFrom<ProcessResponseBody> for GreeterOutputs {
    type Error = Exception;

    fn try_from(value: ProcessResponseBody) -> Result<Self, Self::Error> {
        if let ProcessResponseBody::Requested(buf) = value {
            Ok(GreeterOutputs {
                greeting: String::from_utf8(buf).unwrap(),
            })
        } else {
            Err(Exception::new("500"))
        }
    }
}

#[async_trait::async_trait]
impl Processor for Greeter {
    fn id(&self) -> &'static str {
        "greet"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn process(&self) -> Result<Process> {
        Process::try_new(
            self.id(),
            self.version(),
            &schema_for!(GreeterInputs),
            &schema_for!(GreeterOutputs),
        )
        .map_err(Into::into)
    }

    async fn execute(&self, execute: Execute) -> Result<ProcessResponseBody> {
        let value = serde_json::to_value(execute.inputs).unwrap();
        let inputs: GreeterInputs = serde_json::from_value(value).unwrap();
        Ok(ProcessResponseBody::Requested(
            format!("Hello, {}!\n", inputs.name).as_bytes().to_owned(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use ogcapi_types::processes::Execute;

    use crate::{
        Processor,
        greeter::{Greeter, GreeterInputs, GreeterOutputs},
    };

    #[tokio::test]
    async fn test_greeter() {
        let greeter = Greeter;
        assert_eq!(greeter.id(), "greet");

        println!(
            "Process:\n{}",
            serde_json::to_string_pretty(&greeter.process().unwrap()).unwrap()
        );

        let input = GreeterInputs {
            name: "Greeter".to_string(),
        };

        let execute = Execute {
            inputs: input.execute_input(),
            outputs: GreeterOutputs::execute_output(),
            ..Default::default()
        };

        let output: GreeterOutputs = greeter.execute(execute).await.unwrap().try_into().unwrap();
        assert_eq!(output.greeting, "Hello, Greeter!\n");
    }
}
