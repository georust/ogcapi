use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
    temporal_complex_geometry::TemporalComplexGeometry,
    temporal_primitive_geometry::TemporalPrimitiveGeometry,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, ToSchema)]
#[serde(untagged)]
pub enum TemporalGeometry {
    Primitive(TemporalPrimitiveGeometry),
    Complex(TemporalComplexGeometry),
}

#[cfg(test)]
mod tests {

    use chrono::DateTime;

    use super::*;

    #[test]
    fn moving_complex_geometry_from_json_value() {
        let mut coordinates = vec![];
        let mut datetimes = vec![];
        for i in 0..3 {
            coordinates.push(vec![0., i as f64]);
            datetimes.push(DateTime::from_timestamp(i, 0).unwrap());
        }
        let primitive_geometry =
            TemporalPrimitiveGeometry::new((datetimes, coordinates).try_into().unwrap());
        let geometry = TemporalGeometry::Complex(
            TemporalComplexGeometry::from(vec![primitive_geometry.clone(), primitive_geometry]),
        );
        let deserialized_geometry: TemporalGeometry = serde_json::from_str(
            r#"{
                "type": "MovingGeometryCollection",
                "prisms": [
                    {
                        "type": "MovingPoint",
                        "coordinates": [[0.0, 0.0],[0.0, 1.0],[0.0, 2.0]],
                        "datetimes": ["1970-01-01T00:00:00Z", "1970-01-01T00:00:01Z", "1970-01-01T00:00:02Z"]
                    },
                    {
                        "type": "MovingPoint",
                        "coordinates": [[0.0, 0.0],[0.0, 1.0],[0.0, 2.0]],
                        "datetimes": ["1970-01-01T00:00:00Z", "1970-01-01T00:00:01Z", "1970-01-01T00:00:02Z"]
                    }
                ]
            }"#,
        )
        .unwrap();
        assert_eq!(geometry, deserialized_geometry);
    }

    #[test]
    fn moving_primitive_geometry_from_json_value() {
        let mut coordinates = vec![];
        let mut datetimes = vec![];
        for i in 0..3 {
            coordinates.push(vec![0., i as f64]);
            datetimes.push(DateTime::from_timestamp(i, 0).unwrap());
        }
        let geometry: TemporalGeometry = TemporalGeometry::Primitive(
            TemporalPrimitiveGeometry::new((datetimes, coordinates).try_into().unwrap()),
        );
        let deserialized_geometry: TemporalGeometry = serde_json::from_str(
            r#"{
                "type": "MovingPoint",
                "coordinates": [[0.0, 0.0],[0.0, 1.0],[0.0, 2.0]],
                "datetimes": ["1970-01-01T00:00:00Z", "1970-01-01T00:00:01Z", "1970-01-01T00:00:02Z"]
            }"#,
        )
        .unwrap();
        assert_eq!(geometry, deserialized_geometry);
    }
}
