use serde::{Deserialize, Serialize};

use crate::common::Links;

use super::DescriptionType;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSummary {
    pub id: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub job_control_options: Vec<JobControlOptions>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_transmission: Vec<TransmissionMode>,
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
