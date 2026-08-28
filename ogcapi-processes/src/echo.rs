use std::collections::HashMap;

use anyhow::Result;
use ogcapi_types::processes::{
    Execute, ExecuteResult, ExecuteResults, Format, InlineOrRefData, InputValueNoObject,
    JobControlOptions, Output, Process, ProcessSummary, TransmissionMode,
    description::{DescriptionType, InputDescription, OutputDescription},
};
use schemars::{JsonSchema, generate::SchemaSettings};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::Processor;

/// Echo is a simple process that echoes back the inputs it receives.
/// It is used to verify that the OGC API Processes implementation is working correctly.
///
/// Definition: <https://docs.ogc.org/is/18-062r2/18-062r2.html#_443805da-dfcc-84bd-1820-4a41a69f629a>
#[derive(Clone)]
pub struct Echo;

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct StringInput(String);

#[derive(Debug)]
pub struct EchoParams {
    inputs: EchoInputs,
    requested_outputs: EchoRequestedOutputs,
}

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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EchoRequestedOutputs {
    pub string_output: bool,
    pub double_output: bool,
}

impl TryFrom<&Execute> for EchoRequestedOutputs {
    type Error = anyhow::Error;

    fn try_from(execute: &Execute) -> Result<Self> {
        if execute.outputs.is_empty() {
            return Ok(Self {
                string_output: true,
                double_output: true,
            });
        }

        let mut requested_outputs = Self::default();

        for (output_name, output) in &execute.outputs {
            if output.format.is_some() {
                anyhow::bail!("Custom output formats are not supported in echo process");
            }

            if !matches!(output.transmission_mode, TransmissionMode::Value) {
                anyhow::bail!("Only 'value' transmission mode is supported in echo process");
            }

            match output_name.as_str() {
                "stringOutput" => requested_outputs.string_output = true,
                "doubleOutput" => requested_outputs.double_output = true,
                _ => {
                    anyhow::bail!(
                        "Requested output '{output_name}' is not available in echo process"
                    );
                }
            }
        }

        Ok(requested_outputs)
    }
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, Clone)]
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

impl TryFrom<EchoOutputs> for ExecuteResults {
    type Error = anyhow::Error;

    fn try_from(value: EchoOutputs) -> Result<Self, Self::Error> {
        let outputs = value.compute_output_metadata();
        Ok(value.to_execute_results(&outputs))
    }
}

impl EchoOutputs {
    #[must_use]
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

    fn to_execute_results(&self, outputs: &HashMap<String, Output>) -> ExecuteResults {
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
    type Input = EchoParams;
    type Output = EchoOutputs;

    fn id(&self) -> &'static str {
        "echo"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    async fn process(&self) -> Result<Process> {
        let mut settings = SchemaSettings::default();
        settings.meta_schema = None;

        let mut generator = settings.into_generator();
        Ok(Process {
            summary: ProcessSummary {
                id: self.id().to_string(),
                version: self.version().to_string(),
                description: DescriptionType {
                    title: Some("Echo Process".to_string()),
                    description: Some(
                        "A simple process that echoes back the inputs it receives.".to_string(),
                    ),
                    ..Default::default()
                },
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
                links: vec![],
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
            outputs: HashMap::from([
                (
                    "stringOutput".to_string(),
                    OutputDescription {
                        description_type: DescriptionType::default(),
                        schema: generator.root_schema_for::<StringInput>().to_value(),
                    },
                ),
                (
                    "doubleOutput".to_string(),
                    OutputDescription {
                        description_type: DescriptionType::default(),
                        schema: generator.root_schema_for::<f64>().to_value(),
                    },
                ),
            ]),
        })
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn parse(&self, execute: Execute) -> Result<Self::Input> {
        Ok(EchoParams {
            inputs: serde_json::from_value(serde_json::to_value(&execute.inputs)?)?,
            requested_outputs: EchoRequestedOutputs::try_from(&execute)?,
        })
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn execute(&self, params: Self::Input) -> Result<Self::Output> {
        let EchoParams {
            inputs,
            requested_outputs,
        } = params;
        if let Some(pause_duration) = inputs.pause {
            tokio::time::sleep(std::time::Duration::from_secs(pause_duration)).await;
        }

        Ok(EchoOutputs {
            string_output: inputs
                .string_input
                .filter(|_| requested_outputs.string_output),
            double_output: inputs
                .double_input
                .filter(|_| requested_outputs.double_output),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ogcapi_types::processes::{Input, Output};

    #[tokio::test]
    async fn test_string_value_sync() {
        let echo = Echo;
        assert_eq!(echo.id(), "echo");

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

        let output = echo
            .execute(echo.parse(execute).await.unwrap())
            .await
            .unwrap();
        let results: ExecuteResults = output.try_into().unwrap();

        assert_eq!(results.len(), 1);

        assert_eq!(
            results["stringOutput"].data,
            InlineOrRefData::InputValueNoObject(InputValueNoObject::String("testtest".to_string()))
        );
    }

    #[tokio::test]
    async fn test_multi_value_sync() {
        let echo = Echo;
        assert_eq!(echo.id(), "echo");

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

        let output = echo
            .execute(echo.parse(execute).await.unwrap())
            .await
            .unwrap();
        let results: ExecuteResults = output.try_into().unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(
            results["stringOutput"].data,
            InlineOrRefData::InputValueNoObject(InputValueNoObject::String("testtest".to_string()))
        );
        assert_eq!(
            results["doubleOutput"].data,
            InlineOrRefData::InputValueNoObject(InputValueNoObject::Number(42.0))
        );
    }
}
