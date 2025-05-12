use serde_json;

use serde::de::{self, DeserializeOwned, Error};

use serde::Serialize;

use geojson::JsonValue;

use geojson::JsonObject;

pub(crate) fn expect_type(value: &mut JsonObject) -> Result<String, serde_json::Error> {
    let prop = expect_property(value, "type")?;
    expect_string(prop)
}

pub(crate) fn expect_vec(value: JsonValue) -> Result<Vec<JsonValue>, serde_json::Error> {
    match value {
        JsonValue::Array(s) => Ok(s),
        _ => Err(Error::invalid_type(
            de::Unexpected::Other("type"),
            &"an array",
        )),
    }
}

pub(crate) fn expect_string(value: JsonValue) -> Result<String, serde_json::Error> {
    match value {
        JsonValue::String(s) => Ok(s),
        _ => Err(Error::invalid_type(
            de::Unexpected::Other("type"),
            &"a string",
        )),
    }
}

pub(crate) fn expect_property(
    obj: &mut JsonObject,
    name: &'static str,
) -> Result<JsonValue, serde_json::Error> {
    match obj.remove(name) {
        Some(v) => Ok(v),
        None => Err(Error::missing_field(name)),
    }
}

pub(crate) fn expect_named_vec(
    value: &mut JsonObject,
    name: &'static str,
) -> Result<Vec<JsonValue>, serde_json::Error> {
    let prop = expect_property(value, name)?;
    expect_vec(prop)
}

pub(crate) fn deserialize_iter<T: Serialize + DeserializeOwned>(
    json_vec: Vec<JsonValue>,
) -> impl Iterator<Item = T> {
    json_vec
        .into_iter()
        .map(|v: JsonValue| serde_json::from_value(v).unwrap())
}
