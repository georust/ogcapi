use std::collections::HashMap;

use crate::Processor;
use anyhow::Result;
use ogcapi_types::{
    common::Link,
    processes::{
        Execute, ExecuteResult, Format, InlineOrRefData, InputValueNoObject, JobControlOptions,
        Output, Process, ProcessSummary, TransmissionMode,
        description::{DescriptionType, InputDescription, OutputDescription},
    },
};
use schemars::{JsonSchema, generate::SchemaSettings};
use serde::{Deserialize, Serialize};

/// Echo is a simple process that echoes back the inputs it receives.
/// It is used to verify that the OGC API Processes implementation is working correctly.
///
/// Definition: https://docs.ogc.org/is/18-062r2/18-062r2.html#_443805da-dfcc-84bd-1820-4a41a69f629a
#[derive(Clone)]
pub struct Echo;

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct StringInput(String);

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EchoInputs {
    pub string_input: Option<String>,
    // pub measure_input: Option<MeasureInput>,
    // pub date_input: Option<String>,
    pub double_input: Option<f64>,
    // pub array_input: Option<Vec<i32>>,
    // pub complex_object_input: Option<ComplexObjectInput>,
    // pub geometry_input: Option<Vec<String>>,
    // pub bounding_box_input: Option<BoundingBoxInput>,
    // pub images_input: Option<Vec<String>>,
    // pub feature_collection_input: Option<String>,
    pub pause: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EchoOutputs {
    pub string_output: Option<String>,
    // pub measure_output: Option<MeasureOutput>,
    // pub date_output: Option<String>,
    pub double_output: Option<f64>,
    // pub array_output: Option<Vec<i32>>,
    // pub complex_object_output: Option<ComplexObjectInput>,
    // pub geometry_output: Option<Vec<String>>,
    // pub bounding_box_output: Option<BoundingBoxInput>,
    // pub images_output: Option<Vec<String>>,
    // pub feature_collection_output: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
#[serde(untagged)]
pub enum StringOutput {
    Value1(String),
    Value2(String),
    Value3(String),
}

impl EchoOutputs {
    pub fn compute_output_metadata(&self) -> HashMap<String, Output> {
        let mut outputs = HashMap::new();

        if self.string_output.is_some() {
            outputs.insert(
                "stringOutput".to_string(),
                Output {
                    format: Some(Format {
                        media_type: Some("text/plain".to_string()),
                        encoding: Some("utf8".to_string()),
                        schema: None,
                    }),
                    transmission_mode: TransmissionMode::Value,
                },
            );
        }

        if self.double_output.is_some() {
            outputs.insert(
                "doubleOutput".to_string(),
                Output {
                    format: Some(Format {
                        media_type: Some("text/plain".to_string()),
                        encoding: Some("utf8".to_string()),
                        schema: None,
                    }),
                    transmission_mode: TransmissionMode::Value,
                },
            );
        }

        // TODO: implement for other types

        outputs
    }

    fn to_execute_results(
        &self,
        outputs: &HashMap<String, Output>,
    ) -> HashMap<String, ExecuteResult> {
        let mut execute_results = HashMap::with_capacity(outputs.len());

        if let Some(string_output) = &self.string_output
            && let Some(string_output_meta) = outputs.get("stringOutput")
        {
            execute_results.insert(
                "stringOutput".to_string(),
                ExecuteResult {
                    output: string_output_meta.clone(),
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::String(
                        string_output.clone(),
                    )),
                },
            );
        }

        if let Some(double_output) = &self.double_output
            && let Some(double_output_meta) = outputs.get("doubleOutput")
        {
            execute_results.insert(
                "doubleOutput".to_string(),
                ExecuteResult {
                    output: double_output_meta.clone(),
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::Number(
                        *double_output,
                    )),
                },
            );
        }

        execute_results
    }
}

#[async_trait::async_trait]
impl Processor for Echo {
    fn id(&self) -> &'static str {
        "echo"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn process(&self) -> Result<Process> {
        let mut settings = SchemaSettings::default();
        settings.meta_schema = None;

        let mut generator = settings.into_generator();
        Ok(Process {
            summary: ProcessSummary {
                id: self.id().to_string(),
                version: self.version().to_string(),
                job_control_options: vec![
                    JobControlOptions::SyncExecute,
                    JobControlOptions::AsyncExecute,
                    // TODO: implement "dismiss extension"
                    // JobControlOptions::Dismiss,
                ],
                output_transmission: vec![
                    TransmissionMode::Value,
                    // TODO: implement reference mode
                    // TransmissionMode::Reference,
                ],
                links: vec![
                    Link::new(
                        format!("./{}/execution", self.id()),
                        "http://www.opengis.net/def/rel/ogc/1.0/execute",
                    )
                    .title("Execution endpoint"),
                ],
            },
            inputs: HashMap::from([
                (
                    "stringInput".to_string(),
                    InputDescription {
                        description_type: DescriptionType {
                            title: Some("String Literal Input Example".to_string()),
                            description: Some(
                                "This is an example of a STRING literal input.".to_string(),
                            ),
                            ..Default::default()
                        },
                        schema: generator.root_schema_for::<StringInput>().to_value(),
                        ..Default::default()
                    },
                ),
                (
                    "doubleInput".to_string(),
                    InputDescription {
                        description_type: DescriptionType {
                            title: Some("Double Literal Input Example".to_string()),
                            description: Some(
                                "This is an example of a DOUBLE literal input.".to_string(),
                            ),
                            ..Default::default()
                        },
                        schema: generator.root_schema_for::<f64>().to_value(),
                        ..Default::default()
                    },
                ),
                (
                    "pause".to_string(),
                    InputDescription {
                        description_type: DescriptionType {
                            title: Some("Pause Duration".to_string()),
                            description: Some(
                                "Optional pause duration in seconds before responding.".to_string(),
                            ),
                            ..Default::default()
                        },
                        schema: generator.root_schema_for::<u64>().to_value(),
                        ..Default::default()
                    },
                ),
            ]),
            outputs: HashMap::from([(
                "stringOutput".to_string(),
                OutputDescription {
                    description_type: DescriptionType::default(),
                    schema: generator.root_schema_for::<StringInput>().to_value(),
                },
            )]),
        })
    }

    async fn execute(&self, execute: Execute) -> Result<HashMap<String, ExecuteResult>> {
        let value = serde_json::to_value(execute.inputs)?;
        let inputs: EchoInputs = serde_json::from_value(value)?;

        if let Some(pause_duration) = inputs.pause {
            tokio::time::sleep(std::time::Duration::from_secs(pause_duration)).await;
        }

        let output_values = EchoOutputs {
            string_output: inputs.string_input,
            double_output: inputs.double_input,
        };

        // validate requested outputs
        if !execute.outputs.is_empty() {
            for (output_name, output) in &execute.outputs {
                if !["stringOutput", "doubleOutput"].contains(&output_name.as_str()) {
                    anyhow::bail!(
                        "Requested output '{}' is not available in echo process",
                        output_name
                    );
                }

                if output.format.is_some() {
                    anyhow::bail!("Custom output formats are not supported in echo process");
                }

                if !matches!(output.transmission_mode, TransmissionMode::Value) {
                    anyhow::bail!("Only 'value' transmission mode is supported in echo process");
                }
            }
        }

        let all_outputs = output_values.compute_output_metadata();
        let outputs = if execute.outputs.is_empty() {
            all_outputs
        } else {
            let mut outputs = execute.outputs;
            for (name, output) in &mut outputs {
                let Some(default_output) = all_outputs.get(name) else {
                    anyhow::bail!(
                        "Requested output '{}' is not available in echo process",
                        name
                    );
                };
                if output.format.is_none() {
                    output.format = default_output.format.clone();
                }
            }
            outputs
        };

        Ok(output_values.to_execute_results(&outputs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ogcapi_types::processes::Input;

    #[tokio::test]
    async fn test_string_value_sync() {
        let echo = Echo;
        assert_eq!(echo.id(), "echo");

        eprintln!(
            "Process:\n{}",
            serde_json::to_string_pretty(&echo.process().unwrap()).unwrap()
        );

        let execute = Execute {
            inputs: HashMap::from([(
                "stringInput".to_string(),
                Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                    InputValueNoObject::String("testtest".to_string()),
                )),
            )]),
            outputs: HashMap::from([(
                "stringOutput".to_string(),
                Output {
                    format: None,
                    transmission_mode: TransmissionMode::Value,
                },
            )]),
            ..Default::default()
        };

        let output = echo.execute(execute).await.unwrap();

        assert_eq!(output.len(), 1);

        assert_eq!(
            output["stringOutput"].data,
            InlineOrRefData::InputValueNoObject(InputValueNoObject::String("testtest".to_string()))
        );
    }

    #[tokio::test]
    async fn test_multi_value_sync() {
        let echo = Echo;
        assert_eq!(echo.id(), "echo");

        eprintln!(
            "Process:\n{}",
            serde_json::to_string_pretty(&echo.process().unwrap()).unwrap()
        );

        let execute = Execute {
            inputs: HashMap::from([
                (
                    "stringInput".to_string(),
                    Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                        InputValueNoObject::String("testtest".to_string()),
                    )),
                ),
                (
                    "doubleInput".to_string(),
                    Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                        InputValueNoObject::Number(42.0),
                    )),
                ),
            ]),
            outputs: HashMap::from([
                (
                    "stringOutput".to_string(),
                    Output {
                        format: None,
                        transmission_mode: TransmissionMode::Value,
                    },
                ),
                (
                    "doubleOutput".to_string(),
                    Output {
                        format: None,
                        transmission_mode: TransmissionMode::Value,
                    },
                ),
            ]),
            ..Default::default()
        };

        let output = echo.execute(execute).await.unwrap();

        assert_eq!(output.len(), 2);
        assert_eq!(
            output["stringOutput"].data,
            InlineOrRefData::InputValueNoObject(InputValueNoObject::String("testtest".to_string()))
        );
        assert_eq!(
            output["doubleOutput"].data,
            InlineOrRefData::InputValueNoObject(InputValueNoObject::Number(42.0))
        );
    }
}
