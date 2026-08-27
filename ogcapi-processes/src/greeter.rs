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

impl TryFrom<GreeterOutputs> for ExecuteResults {
    type Error = anyhow::Error;

    fn try_from(value: GreeterOutputs) -> Result<Self, Self::Error> {
        Ok(HashMap::from([(
            "greeting".to_string(),
            ExecuteResult {
                data: InlineOrRefData::InputValueNoObject(InputValueNoObject::String(
                    value.greeting,
                )),
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

#[async_trait::async_trait]
impl Processor for Greeter {
    type Input = GreeterInputs;
    type Output = GreeterOutputs;

    fn id(&self) -> &'static str {
        "greet"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    async fn process(&self) -> Result<Process> {
        Ok(Process {
            summary: ProcessSummary {
                id: self.id().to_string(),
                version: self.version().to_string(),
                description: DescriptionType {
                    title: Some("Greeter".to_string()),
                    description: Some(
                        "A simple process that takes a name as input and returns a greeting."
                            .to_string(),
                    ),
                    ..Default::default()
                },
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

    async fn parse(&self, execute: Execute) -> Result<Self::Input> {
        for output_name in execute.outputs.keys() {
            if output_name != "greeting" {
                return Err(anyhow::anyhow!(
                    "unsupported output requested for Greeter: '{output_name}'"
                ));
            }
        }

        let value = serde_json::to_value(execute.inputs)?;
        Ok(serde_json::from_value(value)?)
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(GreeterOutputs {
            greeting: format!("Hello, {}!\n", input.name),
        })
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
            serde_json::to_string_pretty(&greeter.process().await.unwrap()).unwrap()
        );

        let input = GreeterInputs {
            name: "Greeter".to_string(),
        };

        let execute = Execute {
            inputs: input.execute_input(),
            outputs: HashMap::from([(
                "greeting".to_string(),
                Output {
                    format: None,
                    transmission_mode: TransmissionMode::Value,
                },
            )]),
            ..Default::default()
        };

        let output = greeter
            .execute(greeter.parse(execute).await.unwrap())
            .await
            .unwrap();
        let results: ExecuteResults = output.try_into().unwrap();

        let ExecuteResult { data, output: _ } = results.get("greeting").unwrap();
        let InlineOrRefData::InputValueNoObject(InputValueNoObject::String(greeting)) = data else {
            panic!("Unexpected output data type");
        };

        assert_eq!(greeting, "Hello, Greeter!\n");
    }
}
