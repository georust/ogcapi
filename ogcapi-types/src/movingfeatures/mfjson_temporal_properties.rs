use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize, Serializer, ser::Error};
use serde_json::json;
use utoipa::ToSchema;

use super::temporal_property::Interpolation;

/// MF-JSON TemporalProperties
///
/// A TemporalProperties object is a JSON array of ParametricValues objects that groups a collection of dynamic
/// non-spatial attributes and its parametric values with time.
///
/// See [7.2.2 MF-JSON TemporalProperties](https://docs.ogc.org/is/19-045r3/19-045r3.html#tproperties)
///
/// Opposed to [TemporalProperty](super::temporal_property::TemporalProperty) values for all
/// represented properties are all measured at the same points in time.
#[derive(Deserialize, Debug, Clone, PartialEq, ToSchema)]
pub struct MFJsonTemporalProperties {
    datetimes: Vec<DateTime<FixedOffset>>,
    #[serde(flatten)]
    values: HashMap<String, ParametricValues>,
}

impl MFJsonTemporalProperties {
    pub fn new(
        datetimes: Vec<DateTime<FixedOffset>>,
        values: HashMap<String, ParametricValues>,
    ) -> Result<Self, &'static str> {
        let dt_len = datetimes.len();
        if values.values().any(|property| property.len() != dt_len) {
            Err("all values and datetimes must be of same length")
        } else {
            Ok(Self { datetimes, values })
        }
    }

    pub fn datetimes(&self) -> &[DateTime<FixedOffset>] {
        &self.datetimes
    }

    pub fn values(&self) -> &HashMap<String, ParametricValues> {
        &self.values
    }
}

impl Serialize for MFJsonTemporalProperties {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let dt_len = self.datetimes.len();
        if self
            .values
            .values()
            .any(|property| property.len() != dt_len)
        {
            Err(S::Error::custom(
                "all values and datetimes must be of same length",
            ))
        } else {
            let value = json!(self);
            value.serialize(serializer)
        }
    }
}

/// A ParametricValues object is a JSON object that represents a collection of parametric values of dynamic non-spatial
/// attributes that are ascertained at the same times. A parametric value may be a time-varying measure, a sequence of
/// texts, or a sequence of images. Even though the parametric value may depend on the spatiotemporal location,
/// MF-JSON Prism only considers the temporal dependencies of their changes of value.
///
/// See [7.2.2.1 MF-JSON ParametricValues](https://docs.ogc.org/is/19-045r3/19-045r3.html#pvalues)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ToSchema)]
#[serde(tag = "type")]
pub enum ParametricValues {
    /// The "values" member contains any numeric values.
    Measure {
        values: Vec<f64>,
        /// Allowed Interpolations: Discrete, Step, Linear, Regression
        interpolation: Option<Interpolation>,
        description: Option<f64>,
        /// The "form" member is optional and its value is a JSON string as a common code (3 characters) described in
        /// the [Code List Rec 20 by the UN Centre for Trade Facilitation and Electronic Business (UN/CEFACT)](https://www.unece.org/uncefact/codelistrecs.html) or a
        /// URL specifying the unit of measurement. This member is applied only for a temporal property whose value
        /// type is Measure.
        form: Option<String>,
    },
    /// The "values" member contains any strings.
    Text {
        values: Vec<String>,
        /// Allowed Interpolations: Discrete, Step
        // TODO enforce?
        interpolation: Option<Interpolation>,
        description: Option<String>,
    },
    /// The "values" member contains Base64 strings converted from images or URLs to address images.
    Image {
        values: Vec<String>,
        /// Allowed Interpolations: Discrete, Step
        // TODO enforce?
        interpolation: Option<Interpolation>,
        description: Option<String>,
    },
}

impl ParametricValues {
    fn len(&self) -> usize {
        match self {
            Self::Measure { values, .. } => values.len(),
            Self::Text { values, .. } => values.len(),
            Self::Image { values, .. } => values.len(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn serde_mfjson_temporal_properties() {
        // https://developer.ogc.org/api/movingfeatures/index.html#tag/TemporalProperty/operation/insertTemporalProperty
        let tp_json = r#"[
          {
            "datetimes": [
              "2011-07-14T22:01:01.450Z",
              "2011-07-14T23:01:01.450Z",
              "2011-07-15T00:01:01.450Z"
            ],
            "length": {
              "type": "Measure",
              "form": "http://qudt.org/vocab/quantitykind/Length",
              "values": [
                1,
                2.4,
                1
              ],
              "interpolation": "Linear"
            },
            "discharge": {
              "type": "Measure",
              "form": "MQS",
              "values": [
                3,
                4,
                5
              ],
              "interpolation": "Step"
            }
          },
          {
            "datetimes": [
              "2011-07-14T22:01:01.450Z",
              "2011-07-14T23:01:01.450Z"
            ],
            "camera": {
              "type": "Image",
              "values": [
                "http://www.opengis.net/spec/movingfeatures/json/1.0/prism/example/image1",
                "iVBORw0KGgoAAAANSUhEU......"
              ],
              "interpolation": "Discrete"
            },
            "labels": {
              "type": "Text",
              "values": [
                "car",
                "human"
              ],
              "interpolation": "Discrete"
            }
          }
        ]"#;

        let _: Vec<MFJsonTemporalProperties> =
            serde_json::from_str(tp_json).expect("Failed to parse MF-JSON Temporal Properties");
    }
}
