use std::collections::HashMap;

use anyhow::Result;
use schemars::{JsonSchema, schema_for};
use serde::Deserialize;

use ogcapi_types::processes::{
    Execute, ExecuteResult, ExecuteResults, Format, InlineOrRefData, Input, InputValueNoObject,
    JobControlOptions, Output, Process, ProcessSummary, TransmissionMode,
    description::{DescriptionType, InputDescription, OutputDescription},
};

use crate::Processor;

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

#[async_trait::async_trait]
impl Processor for Greeter {
    fn id(&self) -> &'static str {
        "greet"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn process(&self) -> Result<Process> {
        Ok(Process {
            summary: ProcessSummary {
                id: self.id().to_string(),
                version: self.version().to_string(),
                job_control_options: vec![
                    JobControlOptions::SyncExecute,
                    JobControlOptions::AsyncExecute,
                    JobControlOptions::Dismiss,
                ],
                output_transmission: vec![TransmissionMode::Value, TransmissionMode::Reference],
                links: Vec::new(),
            },
            inputs: HashMap::from([(
                "name".to_string(),
                InputDescription {
                    description_type: DescriptionType::default(),
                    schema: schema_for!(GreeterInputs).to_value(),
                    ..Default::default()
                },
            )]),
            outputs: HashMap::from([(
                "greeting".to_string(),
                OutputDescription {
                    description_type: DescriptionType::default(),
                    schema: schema_for!(GreeterOutputs).to_value(),
                },
            )]),
        })
    }

    async fn execute(&self, execute: Execute) -> Result<ExecuteResults> {
        let value = serde_json::to_value(execute.inputs).unwrap();
        let inputs: GreeterInputs = serde_json::from_value(value).unwrap();
        let greeting = format!("Hello, {}!\n", inputs.name);

        Ok(HashMap::from([(
            "greeting".to_string(),
            ExecuteResult {
                data: InlineOrRefData::InputValueNoObject(InputValueNoObject::String(greeting)),
                output: Output {
                    format: Some(Format {
                        media_type: Some("text/plain".to_string()),
                        encoding: Some("utf8".to_string()),
                        schema: None,
                    }),
                    transmission_mode: TransmissionMode::Value,
                },
            },
        )]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Processor;
    use ogcapi_types::processes::{Execute, ExecuteResult, InlineOrRefData, InputValueNoObject};

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

        let output = greeter.execute(execute).await.unwrap();

        let ExecuteResult { data, output: _ } = output.get("greeting").unwrap();
        let InlineOrRefData::InputValueNoObject(InputValueNoObject::String(greeting)) = data else {
            panic!("Unexpected output data type");
        };

        assert_eq!(greeting, "Hello, Greeter!\n");
    }
}
