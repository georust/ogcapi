use serde::{Deserialize, Serialize};

use super::{crs::Crs, temporal_primitive_geometry::TemporalPrimitiveGeometry, trs::Trs};

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub enum Type {
    #[default]
    MovingGeometryCollection,
}

/// A TemporalComplexGeometry object represents a set of TemporalPrimitiveGeometry objects. When a TemporalGeometry
/// object has a "type" member is "MovingGeometryCollection", the object is specialized as a TemporalComplexGeometry
/// object with one additional mandatory member named "prisms". The value of the "prisms" member is represented by a
/// JSON array of a set of TemporalPrimitiveGeometry instances having at least one element in the array.
///
/// See [7.2.1.2. TemporalComplexGeometry Object](https://docs.ogc.org/is/19-045r3/19-045r3.html#tcomplex)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TemporalComplexGeometry {
    pub r#type: Type,
    pub prisms: Vec<TemporalPrimitiveGeometry>,
    pub crs: Option<Crs>,
    pub trs: Option<Trs>,
}


impl From<Vec<TemporalPrimitiveGeometry>> for TemporalComplexGeometry {

    fn from(value: Vec<TemporalPrimitiveGeometry>) -> Self {
        debug_assert!(!value.is_empty());
        Self {
            r#type: Default::default(),
            prisms: value,
            crs: Default::default(),
            trs: Default::default(),
        }
    }
}
