use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::common::{Bbox, Link, OGC_CRS84};

/// Process execution
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct Execute {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub inputs: HashMap<String, Input>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub outputs: HashMap<String, Output>,
    #[serde(default)]
    pub response: Response,
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(default)]
    pub subscriber: Option<Subscriber>,
}

/// Process execution input
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum Input {
    InlineOrRefData(InlineOrRefData),
    InlineOrRefDataArray(Vec<InlineOrRefData>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum InlineOrRefData {
    InputValueNoObject(InputValueNoObject),
    QualifiedInputValue(QualifiedInputValue),
    Link(Link),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum InputValueNoObject {
    String(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
    Array(Vec<String>),
    // TODO: requires custom serde implementation
    // BinaryInputValue(Vec<u8>), // Undistinguishable from String(String)
    Bbox(BoundingBox), // bbox is actually an object
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct BoundingBox {
    pub bbox: Bbox,
    #[serde(default = "default_crs")]
    pub crs: String,
}

fn default_crs() -> String {
    OGC_CRS84.to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct QualifiedInputValue {
    pub value: InputValue,
    #[serde(flatten)]
    pub format: Format,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum InputValue {
    InputValueNoObject(InputValueNoObject),
    Object(Map<String, Value>),
}

/// Process execution output
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct Output {
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(default)]
    pub format: Option<Format>,
    #[serde(default)]
    pub transmission_mode: TransmissionMode,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct Format {
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(default)]
    pub media_type: Option<String>,
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(default)]
    pub encoding: Option<String>,
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[serde(default)]
    pub schema: Option<Schema>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum Schema {
    String(String),
    Object(Map<String, Value>),
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "utoipa", schema(default = "value"))]
pub enum TransmissionMode {
    #[default]
    Value,
    Reference,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "utoipa", schema(default = "raw"))]
pub enum Response {
    #[default]
    Raw,
    Document,
}

/// Process execution subscriber
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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
    #[cfg_attr(feature = "utoipa", schema(format = Uri))]
    pub success_uri: String,
    #[cfg_attr(feature = "utoipa", schema(nullable = false, format = Uri))]
    #[serde(default)]
    pub in_progress_uri: Option<String>,
    #[cfg_attr(feature = "utoipa", schema(nullable = false, format = Uri))]
    #[serde(default)]
    pub failed_uri: Option<String>,
}

pub type ExecuteResults = HashMap<String, ExecuteResult>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ExecuteResult {
    pub output: Output,
    pub data: InlineOrRefData,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_deserializes_numbers_in_a_stable_way() {
        let json = serde_json::json!({
            "float": 42.0,
            "integer": 42
        });

        let input: HashMap<String, Input> = serde_json::from_value(json.clone()).unwrap();

        assert_eq!(serde_json::to_value(&input).unwrap(), json);
    }
}
