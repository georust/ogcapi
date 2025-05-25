use serde::{Deserialize, Serialize};

use crate::common::Links;

use super::{
    mfjson_temporal_properties::MFJsonTemporalProperties, temporal_property::TemporalProperty,
};

/// A TemporalProperties object consists of the set of [TemporalProperty] or a set of [MFJsonTemporalProperties].
///
/// See [8.8 TemporalProperties](https://docs.ogc.org/is/22-003r3/22-003r3.html#resource-temporalProperties-section)
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemporalProperties {
    pub temporal_properties: TemporalPropertiesValue,
    pub links: Option<Links>,
    pub time_stamp: Option<String>,
    pub number_matched: Option<u64>,
    pub number_returned: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum TemporalPropertiesValue {
    /// [MFJsonTemporalProperties] allows to represent multiple property values all measured at the same points in time.
    MFJsonTemporalProperties(Vec<MFJsonTemporalProperties>),
    /// [TemporalProperty] allows to represent a property value at independent points in time.
    TemporalProperty(Vec<TemporalProperty>),
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        common::Link,
        movingfeatures::{
            mfjson_temporal_properties::ParametricValues, temporal_property::Interpolation,
        },
    };

    use super::*;

    #[test]
    fn serde_temporal_properties() {
        let links: Links = vec![
            Link::new("https://data.example.org/collections/mfc-1/items/mf-1/tproperties","self").mediatype("application/json"),
            Link::new("https://data.example.org/collections/mfc-1/items/mf-1/tproperties&offset=2&limit=2","next").mediatype("application/json"),
        ];

        let temporal_properties = TemporalProperties {
            temporal_properties: TemporalPropertiesValue::MFJsonTemporalProperties(vec![
                MFJsonTemporalProperties {
                    datetimes: vec![
                        // TODO does type actually need to be UTC or could it be FixedOffset
                        // aswell? Converting to UTC loses the information of the original offset!
                        chrono::DateTime::parse_from_rfc3339("2011-07-14T22:01:06.000Z")
                            .unwrap()
                            .into(),
                        chrono::DateTime::parse_from_rfc3339("2011-07-14T22:01:07.000Z")
                            .unwrap()
                            .into(),
                        chrono::DateTime::parse_from_rfc3339("2011-07-14T22:01:08.000Z")
                            .unwrap()
                            .into(),
                    ],
                    values: HashMap::from([
                        (
                            "length".to_string(),
                            ParametricValues::Measure {
                                values: vec![1.0, 2.4, 1.0],
                                interpolation: Some(Interpolation::Linear),
                                description: None,
                                form: Some("http://qudt.org/vocab/quantitykind/Length".to_string()),
                            },
                        ),
                        (
                            "speed".to_string(),
                            ParametricValues::Measure {
                                values: vec![65.0, 70.0, 80.0],
                                interpolation: Some(Interpolation::Linear),
                                form: Some("KMH".to_string()),
                                description: None
                            },
                        ),
                    ]),
                },
            ]),
            links: Some(links),
            time_stamp: Some("2021-09-01T12:00:00Z".into()),
            number_matched: Some(10),
            number_returned: Some(2),
        };

        // https://developer.ogc.org/api/movingfeatures/index.html#tag/TemporalProperty/operation/retrieveTemporalProperties
        let tp_json = r#"{
          "temporalProperties": [
            {
              "datetimes": [
                "2011-07-14T22:01:06.000Z",
                "2011-07-14T22:01:07.000Z",
                "2011-07-14T22:01:08.000Z"
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
              "speed": {
                "type": "Measure",
                "form": "KMH",
                "values": [
                  65,
                  70,
                  80
                ],
                "interpolation": "Linear"
              }
            }
          ],
          "links": [
            {
              "href": "https://data.example.org/collections/mfc-1/items/mf-1/tproperties",
              "rel": "self",
              "type": "application/json"
            },
            {
              "href": "https://data.example.org/collections/mfc-1/items/mf-1/tproperties&offset=2&limit=2",
              "rel": "next",
              "type": "application/json"
            }
          ],
          "timeStamp": "2021-09-01T12:00:00Z",
          "numberMatched": 10,
          "numberReturned": 2
        }"#;
        assert_eq!(temporal_properties, serde_json::from_str(tp_json).unwrap());
    }
}
