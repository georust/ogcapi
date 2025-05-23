use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::temporal_properties::Interpolation;

/// The TemporalPrimitiveValue resource represents the dynamic change of a non-spatial attribute’s value with time. An
/// abbreviated copy of this information is returned for each TemporalPrimitiveValue in the
/// {root}/collections/{collectionId}/items/{mFeatureId}/tproperties/{tPropertyName} response.
///
/// See [8.10. TemporalPrimitiveValue](https://docs.ogc.org/is/22-003r3/22-003r3.html#resource-temporalPrimitiveValue-section)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TemporalPrimitiveValue {
    id: String,
    datetimes: Vec<DateTime<Utc>>,
    values: Vec<Value>,
    interpolation: Interpolation,
}
