use serde::{Deserialize, Serialize};

use super::temporal_primitive_value::TemporalPrimitiveValue;

/// See [8.9. TemporalProperty](https://docs.ogc.org/is/22-003r3/22-003r3.html#resource-temporalProperty-section)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemporalProperty {
    /// An identifier for the resource assigned by an external entity.
    name: String,
    /// A predefined temporal property type.
    r#type: Type,
    value_sequence: Vec<TemporalPrimitiveValue>,
    /// A unit of measure
    form: Option<String>,
    /// A short description
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Type {
    TBoolean,
    TText,
    TInteger,
    TReal,
    TImage,
}
