use serde::{Deserialize, Serialize};
use serde_json::Error;
use utoipa::ToSchema;

use crate::common::Link;

use super::{
    TransmissionMode,
    description::{DescriptionType, InputDescription, MaxOccurs, OutputDescription},
};

/// Process summary
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSummary {
    pub id: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub job_control_options: Vec<JobControlOptions>,
    #[serde(default)]
    pub output_transmission: TransmissionMode,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
    #[serde(flatten)]
    pub description_type: DescriptionType,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum JobControlOptions {
    SyncExecute,
    AsyncExecute,
    Dismiss,
}

/// Information about the available processes
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct ProcessList {
    pub processes: Vec<ProcessSummary>,
    pub links: Vec<Link>,
}

/// Full process description
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct Process {
    #[serde(flatten)]
    pub summary: ProcessSummary,
    #[schema(required = false)]
    pub inputs: InputDescription,
    #[schema(required = false)]
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
                output_transmission: TransmissionMode::default(),
                links: Vec::new(),
                description_type: DescriptionType::default(),
            },
            inputs: InputDescription {
                description_type: DescriptionType::default(),
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
