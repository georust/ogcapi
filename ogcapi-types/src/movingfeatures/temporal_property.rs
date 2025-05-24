use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::common::Links;


/// A temporal property object is a collection of dynamic non-spatial attributes and their temporal values with time. 
/// An abbreviated copy of this information is returned for each TemporalProperty in the 
/// [{root}/collections/{collectionId}/items/{mFeatureId}/tproperties](TemporalProperties) response.
/// The schema for the temporal property object presented in this clause is an extension of the [ParametricValues Object](https://docs.opengeospatial.org/is/19-045r3/19-045r3.html#tproperties) defined in [MF-JSON](https://docs.ogc.org/is/22-003r3/22-003r3.html#OGC_19-045r3).
///
/// See [8.9. TemporalProperty](https://docs.ogc.org/is/22-003r3/22-003r3.html#resource-temporalProperty-section)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TemporalProperty {
    /// An identifier for the resource assigned by an external entity.
    name: String,
    value_sequence: TemporalPropertyValue,
    /// A unit of measure
    form: Option<String>,
    /// A short description
    description: Option<String>,
    links: Option<Links>
}

/// A predefined temporal property type.
///
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum TemporalPropertyValue {
    TBoolean {
        value_sequence: Vec<TemporalPrimitiveValue<bool>>,
    },
    TText {
        value_sequence: Vec<TemporalPrimitiveValue<String>>,
    },
    TInteger {
        value_sequence: Vec<TemporalPrimitiveValue<i64>>,
    },
    TReal {
        value_sequence: Vec<TemporalPrimitiveValue<f64>>,
    },
    TImage {
        value_sequence: Vec<TemporalPrimitiveValue<String>>,
    },
}

/// The TemporalPrimitiveValue resource represents the dynamic change of a non-spatial attribute’s value with time. An
/// abbreviated copy of this information is returned for each TemporalPrimitiveValue in the
/// {root}/collections/{collectionId}/items/{mFeatureId}/tproperties/{tPropertyName} response.
///
/// See [8.10. TemporalPrimitiveValue](https://docs.ogc.org/is/22-003r3/22-003r3.html#resource-temporalPrimitiveValue-section)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct TemporalPrimitiveValue<T> {
    id: String,
    datetimes: Vec<DateTime<Utc>>,
    values: Vec<T>,
    interpolation: Interpolation,
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
    /// between neighboring points in a timeseries, e.g., "http://www.opengis.net/def/timeseries/InterpolationCode/Continuous", 
    /// "http://www.opengis.net/def/timeseries/InterpolationCode/Discontinuous", and etc.
    Url(String),
}
