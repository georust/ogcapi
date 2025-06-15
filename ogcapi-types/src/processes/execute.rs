use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;

use crate::common::{Bbox, Link, OGC_CRS84};

/// Process execution
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Default)]
pub struct Execute {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub inputs: HashMap<String, Input>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub outputs: HashMap<String, Output>,
    #[serde(default)]
    pub response: Response,
    #[schema(nullable = false)]
    #[serde(default)]
    pub subscriber: Option<Subscriber>,
}

/// Process execution input
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(untagged)]
pub enum Input {
    InlineOrRefData(InlineOrRefData),
    InlineOrRefDataArray(Vec<InlineOrRefData>),
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(untagged)]
pub enum InlineOrRefData {
    InputValueNoObject(InputValueNoObject),
    QualifiedInputValue(QualifiedInputValue),
    Link(Link),
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(untagged)]
pub enum InputValueNoObject {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Array(Vec<String>),
    // TODO: requires custom serde implementation
    // BinaryInputValue(Vec<u8>), // Undistinguishable from String(String)
    Bbox(BoundingBox), // bbox is actually an object
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct BoundingBox {
    pub bbox: Bbox,
    #[serde(default = "default_crs")]
    pub crs: String,
}

fn default_crs() -> String {
    OGC_CRS84.to_string()
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct QualifiedInputValue {
    pub value: InputValue,
    #[serde(flatten)]
    pub format: Format,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(untagged)]
pub enum InputValue {
    InputValueNoObject(InputValueNoObject),
    Object(Map<String, Value>),
}

/// Process execution output
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    #[schema(nullable = false)]
    #[serde(default)]
    pub format: Option<Format>,
    #[serde(default)]
    pub transmission_mode: TransmissionMode,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Format {
    #[schema(nullable = false)]
    #[serde(default)]
    pub media_type: Option<String>,
    #[schema(nullable = false)]
    #[serde(default)]
    pub encoding: Option<String>,
    #[schema(nullable = false)]
    #[serde(default)]
    pub schema: Option<Schema>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(untagged)]
pub enum Schema {
    String(String),
    Object(Map<String, Value>),
}

#[derive(Serialize, Deserialize, ToSchema, Default, Debug, Clone)]
#[serde(rename_all = "lowercase")]
#[schema(default = "value")]
pub enum TransmissionMode {
    #[default]
    Value,
    Reference,
}

#[derive(Serialize, Deserialize, ToSchema, Default, Debug, Clone)]
#[serde(rename_all = "lowercase")]
#[schema(default = "raw")]
pub enum Response {
    #[default]
    Raw,
    Document,
}

/// Process execution subscriber
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Subscriber {
    // /// Optional URIs for callbacks for this job.
    // ///
    // /// Support for this parameter is not required and the parameter may be
    // /// removed from the API definition, if conformance class **'callback'**
    // /// is not listed in the conformance declaration under `/conformance`.
    // #[schema(format = Uri)]
    // #[serde(default)]
    // pub description: Option<String>,
    #[schema(format = Uri)]
    pub success_uri: String,
    #[schema(nullable = false, format = Uri)]
    #[serde(default)]
    pub in_progress_uri: Option<String>,
    #[schema(nullable = false, format = Uri)]
    #[serde(default)]
    pub failed_uri: Option<String>,
}
