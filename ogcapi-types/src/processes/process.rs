use serde::{Deserialize, Serialize};
use serde_json::Error;

use crate::common::Links;

use super::description::{
    DescriptionType, InputDescription, MaxOccurs, OutputDescription, ValuePassing,
};

/// Process summary
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSummary {
    pub id: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub job_control_options: Vec<JobControlOptions>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Links,
    #[serde(flatten)]
    pub description_type: DescriptionType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum JobControlOptions {
    SyncExecute,
    AsyncExecute,
    Dismiss,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransmissionMode {
    Value,
    Reference,
}

/// Information about the available processes
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessList {
    pub processes: Vec<ProcessSummary>,
    pub links: Links,
}

/// Full process description
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Process {
    #[serde(flatten)]
    pub summary: ProcessSummary,
    pub inputs: InputDescription,
    pub outputs: OutputDescription,
}

impl Process {
    pub fn try_new<T: Serialize>(
        id: impl ToString,
        version: impl ToString,
        inputs: &T,
        outputs: &T,
    ) -> Result<Self, Error> {
        Ok(Process {
            summary: ProcessSummary {
                id: id.to_string(),
                version: version.to_string(),
                job_control_options: Vec::new(),
                links: Vec::new(),
                description_type: DescriptionType::default(),
            },
            inputs: InputDescription {
                description_type: DescriptionType::default(),
                value_passing: vec![ValuePassing::ByValue],
                min_occurs: 1,
                max_occurs: MaxOccurs::default(),
                schema: serde_json::to_value(inputs)?,
            },
            outputs: OutputDescription {
                description_type: DescriptionType::default(),
                schema: serde_json::to_value(outputs)?,
            },
        })
    }
}
