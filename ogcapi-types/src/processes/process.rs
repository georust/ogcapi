use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Error;
use utoipa::ToSchema;

use crate::common::Link;

use super::{
    TransmissionMode,
    description::{InputDescription, OutputDescription},
};

/// Process summary
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSummary {
    pub id: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub job_control_options: Vec<JobControlOptions>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_transmission: Vec<TransmissionMode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
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
    pub inputs: HashMap<String, InputDescription>,
    #[schema(required = false)]
    pub outputs: HashMap<String, OutputDescription>,
}

impl Process {
    pub fn new(
        id: impl ToString,
        version: impl ToString,
        inputs: HashMap<String, InputDescription>,
        outputs: HashMap<String, OutputDescription>,
    ) -> Result<Self, Error> {
        Ok(Process {
            summary: ProcessSummary {
                id: id.to_string(),
                version: version.to_string(),
                job_control_options: Vec::new(),
                output_transmission: Vec::new(),
                links: Vec::new(),
            },
            inputs,
            outputs,
        })
    }
}
