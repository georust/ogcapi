use super::{
    TransmissionMode,
    description::{InputDescription, OutputDescription},
};
use crate::{common::Link, processes::description::DescriptionType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Process summary
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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
    #[serde(flatten)]
    pub description: DescriptionType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "kebab-case")]
pub enum JobControlOptions {
    SyncExecute,
    AsyncExecute,
    Dismiss,
}

/// Information about the available processes
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ProcessList {
    pub processes: Vec<ProcessSummary>,
    pub links: Vec<Link>,
}

/// Full process description
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Process {
    #[serde(flatten)]
    pub summary: ProcessSummary,
    #[cfg_attr(feature = "utoipa", schema(required = false))]
    pub inputs: HashMap<String, InputDescription>,
    #[cfg_attr(feature = "utoipa", schema(required = false))]
    pub outputs: HashMap<String, OutputDescription>,
}
