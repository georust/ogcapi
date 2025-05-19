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
/// See (7.2.1.2. TemporalComplexGeometry Object)[https://docs.ogc.org/is/19-045r3/19-045r3.html#tcomplex]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TemporalComplexGeometry{
    r#type: Type,
    prisms: Prisms,
    crs: Option<Crs>,
    trs: Option<Trs>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(try_from = "PrismsUnchecked")]
pub struct Prisms(Vec<TemporalPrimitiveGeometry>);

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct PrismsUnchecked(Vec<TemporalPrimitiveGeometry>);

impl TryFrom<Vec<TemporalPrimitiveGeometry>> for TemporalComplexGeometry{
    type Error = &'static str;

    fn try_from(value: Vec<TemporalPrimitiveGeometry>) -> Result<Self, Self::Error> {
        Ok(Self{
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
        if !value.0.is_empty(){
            Ok(Prisms(value.0))
        }else{
            Err("Prisms must have at least one value")
        }
    }
}

