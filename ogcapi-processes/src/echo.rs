use std::collections::HashMap;

use crate::{ProcessResponseBody, Processor};
use anyhow::Result;
use ogcapi_types::processes::{Execute, Format, Output, Process, TransmissionMode};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

/// Echo is a simple process that echoes back the inputs it receives.
/// It is used to verify that the OGC API Processes implementation is working correctly.
///
/// Definition: https://docs.ogc.org/is/18-062r2/18-062r2.html#_443805da-dfcc-84bd-1820-4a41a69f629a
#[derive(Clone)]
pub struct Echo;

#[derive(Deserialize, Debug, JsonSchema)]
pub struct EchoInputs {
    pub string_input: Option<String>,
    pub measure_input: Option<MeasureInput>,
    pub date_input: Option<String>,
    pub double_input: Option<f64>,
    pub array_input: Option<Vec<i32>>,
    pub complex_object_input: Option<ComplexObjectInput>,
    pub geometry_input: Option<Vec<String>>,
    pub bounding_box_input: Option<BoundingBoxInput>,
    pub images_input: Option<Vec<String>>,
    pub feature_collection_input: Option<String>,
}

#[derive(Clone, Deserialize, Serialize, Debug, JsonSchema)]
pub struct MeasureInput {
    pub measurement: f64,
    pub uom: String,
    pub reference: Option<String>,
}

#[derive(Clone, Deserialize, Serialize, Debug, JsonSchema)]
pub struct ComplexObjectInput {
    pub property1: String,
    pub property2: Option<String>,
    pub property3: Option<f64>,
    pub property4: Option<String>,
    pub property5: bool,
}

#[derive(Clone, Deserialize, Serialize, Debug, JsonSchema)]
pub struct BoundingBoxInput {
    pub bbox: Vec<f64>,
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct EchoOutputs {
    pub string_input: Option<String>,
    pub measure_input: Option<MeasureInput>,
    pub date_input: Option<String>,
    pub double_input: Option<f64>,
    pub array_input: Option<Vec<i32>>,
    pub complex_object_input: Option<ComplexObjectInput>,
    pub geometry_input: Option<Vec<String>>,
    pub bounding_box_input: Option<BoundingBoxInput>,
    pub images_input: Option<Vec<String>>,
    pub feature_collection_input: Option<String>,
}

impl EchoOutputs {
    pub fn all_outputs() -> HashMap<String, Output> {
        HashMap::from([
            (
                "string_input".to_string(),
                Output {
                    format: Some(Format {
                        media_type: Some("text/plain".to_string()),
                        encoding: Some("utf8".to_string()),
                        schema: None,
                    }),
                    transmission_mode: TransmissionMode::Value,
                },
            ),
            // TODO: implement (cf. "Multiple raw outputs as multipart/related not implemented yet!")
            // (
            //     "double_input".to_string(),
            //     Output {
            //         format: Some(Format {
            //             media_type: Some("text/plain".to_string()),
            //             encoding: Some("utf8".to_string()),
            //             schema: None,
            //         }),
            //         transmission_mode: TransmissionMode::Value,
            //     },
            // ),
        ])
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
        Process::try_new(
            self.id(),
            self.version(),
            &schema_for!(EchoInputs),
            &schema_for!(EchoOutputs),
        )
        .map_err(Into::into)
    }

    async fn execute(&self, execute: Execute) -> Result<ProcessResponseBody> {
        let value = serde_json::to_value(execute.inputs).unwrap();
        let inputs: EchoInputs = serde_json::from_value(value).unwrap();

        let outputs = EchoOutputs {
            string_input: inputs.string_input,
            measure_input: inputs.measure_input,
            date_input: inputs.date_input,
            double_input: inputs.double_input,
            array_input: inputs.array_input,
            complex_object_input: inputs.complex_object_input,
            geometry_input: inputs.geometry_input,
            bounding_box_input: inputs.bounding_box_input,
            images_input: inputs.images_input,
            feature_collection_input: inputs.feature_collection_input,
        };

        let response = serde_json::to_vec(&outputs)?;

        // "If a process is defined as having one or more outputs and the outputs parameter is omitted in an execute request, this SHALL be equivalent to having requested all the defined outputs in the execute request."
        // TODO: add this from processor trait
        let outputs = if execute.outputs.is_empty() {
            EchoOutputs::all_outputs()
        } else {
            execute.outputs
        };

        Ok(ProcessResponseBody::Requested {
            outputs,
            parts: vec![response],
        })
    }
}
