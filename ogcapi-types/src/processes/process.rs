use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use crate::common::Links;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Process {
    #[serde(flatten)]
    pub summary: ProcessSummary,
    pub inputs: InputDescription,
    pub outputs: OutputDescription,
}

/// Information about the available processes
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessList {
    pub processes: Vec<ProcessSummary>,
    pub links: Links,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSummary {
    pub id: String,
    version: String,
    job_control_options: Option<Vec<JobControlOptions>>,
    output_transmission: Option<Vec<TransmissionMode>>,
    #[serde(default)]
    pub links: Links,
    #[serde(flatten)]
    description_type: DescriptionType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DescriptionType {
    title: Option<String>,
    description: Option<String>,
    keywords: Option<Vec<String>>,
    metadata: Option<Vec<Matadata>>,
    parameters: Option<Vec<AdditionalParameter>>,
    role: Option<String>,
    href: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InputDescription {
    #[serde(flatten)]
    description_type: DescriptionType,
    min_occurs: i32,
    max_occurs: MaxOccurs,
    schema: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OutputDescription {
    #[serde(flatten)]
    description_type: DescriptionType,
    schema: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Matadata {
    title: Option<String>,
    role: Option<String>,
    href: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AdditionalParameter {
    name: String,
    value: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
enum JobControlOptions {
    SyncExecute,
    AsyncExecute,
    Dismiss,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
enum TransmissionMode {
    Value,
    Reference,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum MaxOccurs {
    Integer(i32),
    Unbounded(String),
}
