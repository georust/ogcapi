use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::common::{Bbox, Link};

#[derive(Serialize, Deserialize, Debug)]
pub struct Execute {
    inputs: Option<HashMap<String, Input>>,
    outputs: Option<HashMap<String, Output>>,
    #[serde(default)]
    response: Response,
    subscriber: Subscriber,
}

#[derive(Serialize, Deserialize, Debug)]
enum Input {
    InlineOrRefData(InlineOrRefData),
    InlineOrRefDataArray(Vec<InlineOrRefData>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum InlineOrRefData {
    InputValueNoObject(InputValueNoObject),
    QualifiedInputValue(QualifiedInputValue),
    Link(Link),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum InputValueNoObject {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Array(Vec<Value>),
    BinaryInputValue(String),
    Bbox(BoundingBox),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoundingBox {
    bbox: Bbox,
    crs: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QualifiedInputValue {
    value: InputValue,
    #[serde(flatten)]
    format: Format,
}

#[derive(Serialize, Deserialize, Debug)]
enum InputValue {
    InputValueNoObject(InputValueNoObject),
    Object(Map<String, Value>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Output {
    format: Option<Format>,
    #[serde(default)]
    transmission_mode: TransmissionMode,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Format {
    media_type: Option<String>,
    encoding: Option<String>,
    schema: Option<Schema>,
}

#[derive(Serialize, Deserialize, Debug)]
enum Schema {
    String(String),
    Object(Map<String, Value>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum TransmissionMode {
    Value,
    Reference,
}

impl Default for TransmissionMode {
    fn default() -> Self {
        TransmissionMode::Value
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Response {
    Raw,
    Document,
}

impl Default for Response {
    fn default() -> Self {
        Response::Raw
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Subscriber {
    success_uri: String,
    in_progress_uri: Option<String>,
    failed_uri: Option<String>,
}
