use chrono::{DateTime, Utc};
use serde::{ser, Deserialize, Serialize, Serializer};
use serde_json::json;

use crate::common::Links;

/// A temporal property object is a collection of dynamic non-spatial attributes and their temporal values with time.
/// An abbreviated copy of this information is returned for each TemporalProperty in the
/// [{root}/collections/{collectionId}/items/{mFeatureId}/tproperties](super::temporal_properties::TemporalProperties) response.
/// The schema for the temporal property object presented in this clause is an extension of the [ParametricValues Object](https://docs.opengeospatial.org/is/19-045r3/19-045r3.html#tproperties) defined in [MF-JSON](https://docs.ogc.org/is/22-003r3/22-003r3.html#OGC_19-045r3).
///
/// See [8.9. TemporalProperty](https://docs.ogc.org/is/22-003r3/22-003r3.html#resource-temporalProperty-section)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemporalProperty {
    /// An identifier for the resource assigned by an external entity.
    pub name: String,
    #[serde(flatten)]
    pub value_sequence: TemporalPropertyValue,
    /// A unit of measure
    pub form: Option<String>,
    /// A short description
    pub description: Option<String>,
    pub links: Option<Links>,
}

/// A predefined temporal property type.
///
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", content = "valueSequence")]
pub enum TemporalPropertyValue {
    TBoolean(Vec<TemporalPrimitiveValue<bool>>),
    TText(Vec<TemporalPrimitiveValue<String>>),
    TInteger(Vec<TemporalPrimitiveValue<i64>>),
    TReal(Vec<TemporalPrimitiveValue<f64>>),
    TImage(Vec<TemporalPrimitiveValue<String>>),
}

/// The TemporalPrimitiveValue resource represents the dynamic change of a non-spatial attribute’s value with time. An
/// abbreviated copy of this information is returned for each TemporalPrimitiveValue in the
/// {root}/collections/{collectionId}/items/{mFeatureId}/tproperties/{tPropertyName} response.
///
/// See [8.10. TemporalPrimitiveValue](https://docs.ogc.org/is/22-003r3/22-003r3.html#resource-temporalPrimitiveValue-section)
#[derive(Deserialize, Debug, Default, Clone, PartialEq)]
#[serde(try_from = "TemporalPrimitiveValueUnchecked<T>")]
pub struct TemporalPrimitiveValue<T> {
    /// A unique identifier to the temporal primitive value.
    // TODO mandatory according to https://docs.ogc.org/is/22-003r3/22-003r3.html#_overview_13
    // but missing in response sample at https://developer.ogc.org/api/movingfeatures/index.html#tag/TemporalProperty/operation/retrieveTemporalProperty
    pub id: Option<String>,
    /// A sequence of monotonic increasing instants.
    pub datetimes: Vec<DateTime<Utc>>,
    /// A sequence of dynamic values having the same number of elements as “datetimes”.
    // TODO enforce length
    pub values: Vec<T>,
    /// A predefined type for a dynamic value (i.e., one of ‘Discrete’, ‘Step’, ‘Linear’, or ‘Regression’).
    pub interpolation: Interpolation,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct TemporalPrimitiveValueUnchecked<T> {
    id: Option<String>,
    datetimes: Vec<DateTime<Utc>>,
    values: Vec<T>,
    interpolation: Interpolation,
}

impl<T> TryFrom<TemporalPrimitiveValueUnchecked<T>> for TemporalPrimitiveValue<T>{
    type Error = &'static str;

    fn try_from(value: TemporalPrimitiveValueUnchecked<T>) -> Result<Self, Self::Error> {
        if value.values.len() != value.datetimes.len() {
            Err("values and datetimes must be of same length")
        }else{
            Ok(Self{
                id: value.id,
                interpolation: value.interpolation,
                datetimes: value.datetimes, 
                values: value.values
            })
        }
    }
}

impl<T> Serialize for TemporalPrimitiveValue<T>{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.values.len() != self.datetimes.len() {
             Err(ser::Error::custom("values and datetimes must be of same length"))
        }else{
            let value = json!(self);
            value.serialize(serializer) 
        }
        
    }
}

/// See [ParametricValues Object -> "interpolation"](https://docs.opengeospatial.org/is/19-045r3/19-045r3.html#tproperties)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub enum Interpolation {
    /// The sampling of the attribute occurs such that it is not possible to regard the series as continuous; thus,
    /// there is no interpolated value if t is not an element in "datetimes".
    #[default]
    Discrete,
    /// The values are not connected at the end of a subinterval with two successive instants. The value just jumps from
    /// one value to the other at the end of a subinterval.
    Step,
    /// The values are essentially connected and a linear interpolation estimates the value of the property at the
    /// indicated instant during a subinterval.
    Linear,
    /// The value of the attribute at the indicated instant is extrapolated from a simple linear regression model with
    /// the whole values corresponding to the all elements in "datetimes".
    Regression,
    /// For a URL, this standard refers to the [InterpolationCode Codelist](http://docs.opengeospatial.org/is/15-042r3/15-042r3.html#75) defined in [OGC TimeseriesML 1.0](http://docs.opengeospatial.org/is/15-042r3/15-042r3.html)[OGC 15-042r3]
    /// between neighboring points in a timeseries, e.g., "<http://www.opengis.net/def/timeseries/InterpolationCode/Continuous>",
    /// "<http://www.opengis.net/def/timeseries/InterpolationCode/Discontinuous>", and etc.
    Url(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_temporal_property() {
        // https://developer.ogc.org/api/movingfeatures/index.html#tag/TemporalProperty/operation/retrieveTemporalProperty
        let tp_json = r#"{
          "name": "speed",
          "type": "TReal",
          "form": "KMH",
          "valueSequence": [
            {
              "datetimes": [
                "2011-07-15T08:00:00Z",
                "2011-07-15T08:00:01Z",
                "2011-07-15T08:00:02Z"
              ],
              "values": [
                0,
                20,
                50
              ],
              "interpolation": "Linear"
            }
          ]
        }"#;

        let _: TemporalProperty =
            serde_json::from_str(tp_json).expect("Failed to deserialize Temporal Property");
    }
}
