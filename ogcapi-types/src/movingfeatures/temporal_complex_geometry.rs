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
    pub prisms: Prisms,
    pub crs: Option<Crs>,
    pub trs: Option<Trs>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(try_from = "PrismsUnchecked")]
pub struct Prisms(Vec<TemporalPrimitiveGeometry>);

impl Prisms {
    pub fn new(value: Vec<TemporalPrimitiveGeometry>) -> Result<Self, &'static str> {
        if !value.is_empty() {
            Ok(Self(value))
        } else {
            Err("Prisms must have at least one value")
        }
    }

    pub fn push(&mut self, value: TemporalPrimitiveGeometry) {
        self.0.push(value)
    }

    pub fn is_empty(&self) -> bool {
        // this should never be true
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn try_remove(&mut self, id: &str) -> Result<TemporalPrimitiveGeometry, &'static str> {
        if self.len() > 2 {
            self.0
                .pop_if(|tg| tg.id.as_ref().is_some_and(|tg_id| tg_id == id))
                .ok_or("Temporal Geometry not found.")
        } else {
            Err("Prisms must have at least one value. Try to delete the whole prism.")
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct PrismsUnchecked(Vec<TemporalPrimitiveGeometry>);

impl TryFrom<Vec<TemporalPrimitiveGeometry>> for TemporalComplexGeometry {
    type Error = &'static str;

    fn try_from(value: Vec<TemporalPrimitiveGeometry>) -> Result<Self, Self::Error> {
        Ok(Self {
            r#type: Default::default(),
            prisms: PrismsUnchecked(value).try_into()?,
            crs: Default::default(),
            trs: Default::default(),
        })
    }
}

impl TryFrom<PrismsUnchecked> for Prisms {
    type Error = &'static str;

    fn try_from(value: PrismsUnchecked) -> Result<Self, Self::Error> {
        Self::new(value.0)
    }
}
