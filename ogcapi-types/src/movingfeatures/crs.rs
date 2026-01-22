use std::str::FromStr;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common;

/// MF-JSON uses a CRS as described in in (GeoJSON:2008)[https://geojson.org/geojson-spec#coordinate-reference-system-objects]
/// See (7.2.3 CoordinateReferenceSystem Object)[https://docs.ogc.org/is/19-045r3/19-045r3.html#crs]
/// See (6. Overview of Moving features JSON Encodings)[https://docs.ogc.org/is/19-045r3/19-045r3.html#_overview_of_moving_features_json_encodings_informative]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(tag = "type", content = "properties")]
pub enum Crs {
    /// A Named CRS object indicates a coordinate reference system by name. In this case, the value of its "type" member
    /// is the string "Name". The value of its "properties" member is a JSON object containing a "name" member whose
    /// value is a string identifying a coordinate reference system (not JSON null value). The value of "href" and "type"
    /// is a JSON null value. This standard recommends an EPSG[3] code as the value of "name", such as "EPSG::4326."
    ///
    /// See (7.2.3.1 Named CRS)[https://docs.ogc.org/is/19-045r3/19-045r3.html#_7_2_3_1_named_crs]
    Name { name: String },
    /// A linked CRS object has one required member "href" and one optional member "type". The value of the required "href"
    /// member is a dereferenceable URI. The value of the optional "type" member is a string that hints at the format used
    /// to represent CRS parameters at the provided URI. Suggested values are: "Proj4", "OGCWKT", "ESRIWKT", but others can
    /// be used.
    ///
    /// See (7.2.3.2. Linked CRS)[https://docs.ogc.org/is/19-045r3/19-045r3.html#_7_2_3_2_linked_crs]
    Link {
        r#type: Option<String>,
        href: String,
    },
}

impl Default for Crs {
    fn default() -> Self {
        Self::Name {
            // FIXME: Should this be respect 3d?
            name: common::Crs::default2d().to_urn(),
        }
    }
}

impl TryFrom<Crs> for common::Crs {
    type Error = String;

    fn try_from(value: Crs) -> Result<Self, Self::Error> {
        match value {
            Crs::Name { name } => Self::from_str(name.as_str()),
            Crs::Link { href, .. } => Self::from_str(href.as_str()),
        }
    }
}

impl From<common::Crs> for Crs {
    fn from(value: common::Crs) -> Self {
        Self::Name {
            name: value.to_urn(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn serde_json() {
        // TODO this contradicts example from https://developer.ogc.org/api/movingfeatures/index.html#tag/MovingFeatures/operation/retrieveMovingFeatures
        // Example from https://docs.ogc.org/is/19-045r3/19-045r3.html#_7_2_3_1_named_crs
        let trs: Crs = serde_json::from_str(
            r#"
            {
              "type": "Name",
              "properties": {
                "name": "urn:ogc:def:crs:OGC:1.3:CRS84"
              }
            }
        "#,
        )
        .expect("Failed to parse Crs");
        let expected_trs = Crs::default();
        assert_eq!(trs, expected_trs);
    }

    #[test]
    fn into_common_crs() {
        // assert_eq!(common::Crs::try_from(Crs::default()).unwrap(), common::Crs::default());
        assert_eq!(common::Crs::default2d(), Crs::default().try_into().unwrap());

        // assert_eq!(Crs::from(common::Crs::default()), Crs::default());
        assert_eq!(Crs::default(), common::Crs::default2d().into());
    }
}
