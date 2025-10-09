use std::collections::HashMap;

use crate::{ProcessResponseBody, Processor};
use anyhow::Result;
use ogcapi_types::{
    common::Link,
    processes::{
        Execute, Format, InlineOrRefData, InputValueNoObject, JobControlOptions, Output, Process,
        ProcessSummary, Response, Results, TransmissionMode,
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
pub struct EchoInputs {
    pub string_input: Option<String>,
    // pub measure_input: Option<MeasureInput>,
    // pub date_input: Option<String>,
    // pub double_input: Option<f64>,
    // pub array_input: Option<Vec<i32>>,
    // pub complex_object_input: Option<ComplexObjectInput>,
    // pub geometry_input: Option<Vec<String>>,
    // pub bounding_box_input: Option<BoundingBoxInput>,
    // pub images_input: Option<Vec<String>>,
    // pub feature_collection_input: Option<String>,
}

// #[derive(Clone, Deserialize, Serialize, Debug, JsonSchema)]
// pub struct MeasureInput {
//     pub measurement: f64,
//     pub uom: String,
//     pub reference: Option<String>,
// }

// #[derive(Clone, Deserialize, Serialize, Debug, JsonSchema)]
// pub struct ComplexObjectInput {
//     pub property1: String,
//     pub property2: Option<String>,
//     pub property3: Option<f64>,
//     pub property4: Option<String>,
//     pub property5: bool,
// }

// #[derive(Clone, Deserialize, Serialize, Debug, JsonSchema)]
// pub struct BoundingBoxInput {
//     pub bbox: Vec<f64>,
// }

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
#[serde(untagged)]
pub enum StringOutput {
    Value1(String),
    Value2(String),
    Value3(String),
}

// #[derive(Clone, Deserialize, Serialize, Debug, JsonSchema)]
// pub struct MeasureInput {
//     pub measurement: f64,
//     pub uom: String,
//     pub reference: Option<String>,
// }

// #[derive(Clone, Deserialize, Serialize, Debug, JsonSchema)]
// pub struct ComplexObjectInput {
//     pub property1: String,
//     pub property2: Option<String>,
//     pub property3: Option<f64>,
//     pub property4: Option<String>,
//     pub property5: bool,
// }

// #[derive(Clone, Deserialize, Serialize, Debug, JsonSchema)]
// pub struct BoundingBoxInput {
//     pub bbox: Vec<f64>,
// }

// #[derive(Clone, Debug, JsonSchema, Serialize)]
// pub struct EchoOutputs {
//     pub string_input: Option<String>,
//     pub measure_input: Option<MeasureInput>,
//     pub date_input: Option<String>,
//     pub double_input: Option<f64>,
//     pub array_input: Option<Vec<i32>>,
//     pub complex_object_input: Option<ComplexObjectInput>,
//     pub geometry_input: Option<Vec<String>>,
//     pub bounding_box_input: Option<BoundingBoxInput>,
//     pub images_input: Option<Vec<String>>,
//     pub feature_collection_input: Option<String>,
// }

// impl EchoOutputs {
//     pub fn all_outputs() -> HashMap<String, Output> {
//         HashMap::from([
//             (
//                 "stringInput".to_string(),
//                 Output {
//                     format: Some(Format {
//                         media_type: Some("text/plain".to_string()),
//                         encoding: Some("utf8".to_string()),
//                         schema: None,
//                     }),
//                     transmission_mode: TransmissionMode::Value,
//                 },
//             ),
//             // TODO: implement (cf. "Multiple raw outputs as multipart/related not implemented yet!")
//             // (
//             //     "double_input".to_string(),
//             //     Output {
//             //         format: Some(Format {
//             //             media_type: Some("text/plain".to_string()),
//             //             encoding: Some("utf8".to_string()),
//             //             schema: None,
//             //         }),
//             //         transmission_mode: TransmissionMode::Value,
//             //     },
//             // ),
//         ])
//     }
// }

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
                    JobControlOptions::Dismiss,
                ],
                output_transmission: vec![TransmissionMode::Value, TransmissionMode::Reference],
                links: vec![
                    Link::new(
                        "./execute",
                        "http://www.opengis.net/def/rel/ogc/1.0/execute",
                    )
                    .title("Execution endpoint"),
                ],
            },
            inputs: HashMap::from([(
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
            )]),
            outputs: HashMap::from([(
                "stringOutput".to_string(),
                OutputDescription {
                    description_type: DescriptionType::default(),
                    schema: generator.root_schema_for::<StringOutput>().to_value(),
                },
            )]),
        })
    }

    async fn execute(&self, execute: Execute) -> Result<ProcessResponseBody> {
        let value = serde_json::to_value(execute.inputs).unwrap();
        let inputs: EchoInputs = serde_json::from_value(value).unwrap();

        // let outputs = EchoOutputs {
        //     string_input: inputs.string_input,
        //     measure_input: inputs.measure_input,
        //     date_input: inputs.date_input,
        //     double_input: inputs.double_input,
        //     array_input: inputs.array_input,
        //     complex_object_input: inputs.complex_object_input,
        //     geometry_input: inputs.geometry_input,
        //     bounding_box_input: inputs.bounding_box_input,
        //     images_input: inputs.images_input,
        //     feature_collection_input: inputs.feature_collection_input,
        // };

        // let response: Vec<u8> = serde_json::to_vec(&outputs)?;

        match execute.response {
            Response::Raw => Ok(ProcessResponseBody::Requested {
                outputs: HashMap::from([(
                    "stringOutput".to_string(),
                    Output {
                        format: Some(Format {
                            media_type: Some("text/plain".to_string()),
                            encoding: Some("utf8".to_string()),
                            schema: None,
                        }),
                        transmission_mode: TransmissionMode::Value,
                    },
                )]),
                parts: vec![serde_json::to_vec(&inputs).unwrap()],
            }),
            Response::Document => Ok(ProcessResponseBody::Results(Results {
                results: HashMap::from([(
                    "stringOutput".to_owned(),
                    InlineOrRefData::InputValueNoObject(InputValueNoObject::String(
                        inputs.string_input.unwrap_or_default(),
                    )),
                )]),
            })),
        }

        // // "If a process is defined as having one or more outputs and the outputs parameter is omitted in an execute request, this SHALL be equivalent to having requested all the defined outputs in the execute request."
        // // TODO: add this from processor trait
        // let outputs = if execute.outputs.is_empty() {
        //     EchoOutputs::all_outputs()
        // } else {
        //     execute.outputs
        // };

        // Ok(ProcessResponseBody::Requested {
        //     outputs,
        //     parts: vec![response],
        // })
    }
}
