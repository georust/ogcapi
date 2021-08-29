use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;

use crate::common::Links;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Process {
    #[serde(flatten)]
    pub summary: Json<ProcessSummary>,
    pub inputs: Json<InputDescription>,
    pub outputs: Json<OutputDescription>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSummary {
    pub id: String,
    version: String,
    job_control_options: Option<Vec<JobControlOptions>>,
    output_transmission: Option<Vec<TransmissionMode>>,
    pub links: Option<Links>,
    #[serde(flatten)]
    description_type: DescriptionType,
}

#[derive(Serialize, Deserialize, Debug)]
struct DescriptionType {
    title: Option<String>,
    description: Option<String>,
    keywords: Option<Vec<String>>,
    metadata: Option<Vec<Matadata>>,
    parameters: Option<Vec<AdditionalParameter>>,
    role: Option<String>,
    href: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputDescription {
    #[serde(flatten)]
    description_type: DescriptionType,
    min_occurs: i32,
    max_occurs: MaxOccurs,
    schema: Value,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputDescription {
    #[serde(flatten)]
    description_type: DescriptionType,
    schema: Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct Matadata {
    title: Option<String>,
    role: Option<String>,
    href: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AdditionalParameter {
    name: String,
    value: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]

enum JobControlOptions {
    SyncEcecute,
    AsyncEcecute,
    Dismiss,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]

enum TransmissionMode {
    Value,
    Reference,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum MaxOccurs {
    Integer(i32),
    Unbounded(String),
}
