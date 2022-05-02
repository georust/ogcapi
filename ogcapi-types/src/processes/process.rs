use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::common::Links;

use super::{DescriptionType, InputDescription, MaxOccurs, OutputDescription, ProcessSummary};

/// Information about the available processes
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessList {
    pub processes: Vec<ProcessSummary>,
    pub links: Links,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Process {
    #[serde(flatten)]
    pub summary: ProcessSummary,
    pub inputs: InputDescription,
    pub outputs: OutputDescription,
}

impl Process {
    pub fn new(id: impl ToString, version: impl ToString, inputs: &Value, outputs: &Value) -> Self {
        Process {
            summary: ProcessSummary {
                id: id.to_string(),
                version: version.to_string(),
                job_control_options: Vec::new(),
                output_transmission: Vec::new(),
                links: Vec::new(),
                description_type: DescriptionType::default(),
            },
            inputs: InputDescription {
                description_type: DescriptionType::default(),
                min_occurs: 1,
                max_occurs: MaxOccurs::default(),
                schema: inputs.to_owned(),
            },
            outputs: OutputDescription {
                description_type: DescriptionType::default(),
                schema: outputs.to_owned(),
            },
        }
    }
}
