use std::collections::HashMap;

use anyhow::Result;
use schemars::{JsonSchema, schema_for};
use serde::Deserialize;

use ogcapi_types::processes::{
    Execute, Format, InlineOrRefData, Input, InputValueNoObject, Output, Process, Response,
    Results, TransmissionMode,
    description::{DescriptionType, InputDescription, MaxOccurs, OutputDescription},
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

#[async_trait::async_trait]
impl Processor for Greeter {
    fn id(&self) -> &'static str {
        "greet"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn process(&self) -> Result<Process> {
        Process::new(
            self.id(),
            self.version(),
            HashMap::from([(
                "name".to_string(),
                InputDescription {
                    description_type: DescriptionType::default(),
                    min_occurs: 1,
                    max_occurs: MaxOccurs::default(),
                    schema: schema_for!(GreeterInputs).to_value(),
                },
            )]),
            HashMap::from([(
                "greeting".to_string(),
                OutputDescription {
                    description_type: DescriptionType::default(),
                    schema: schema_for!(GreeterOutputs).to_value(),
                },
            )]),
        )
        .map_err(Into::into)
    }

    async fn execute(&self, execute: Execute) -> Result<ProcessResponseBody> {
        let value = serde_json::to_value(execute.inputs).unwrap();
        let inputs: GreeterInputs = serde_json::from_value(value).unwrap();
        let greeting = format!("Hello, {}!\n", inputs.name);
        match execute.response {
            Response::Raw => Ok(ProcessResponseBody::Requested {
                outputs: GreeterOutputs::execute_output(),
                parts: vec![greeting.as_bytes().to_owned()],
            }),
            Response::Document => Ok(ProcessResponseBody::Results(Results {
                results: HashMap::from([(
                    "greeting".to_owned(),
                    InlineOrRefData::InputValueNoObject(InputValueNoObject::String(greeting)),
                )]),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use ogcapi_types::processes::Execute;

    use crate::{
        ProcessResponseBody, Processor,
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

        let output = greeter.execute(execute).await.unwrap();

        let ProcessResponseBody::Requested {
            outputs: _outputs,
            parts,
        } = output
        else {
            panic!()
        };

        assert_eq!(
            String::from_utf8(parts[0].clone()).unwrap(),
            "Hello, Greeter!\n"
        );
    }
}
