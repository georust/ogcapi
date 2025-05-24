use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
// TODO enforce same length of datetimes and values
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MFJsonTemporalProperties {
    datetimes: Vec<DateTime<Utc>>,
    #[serde(flatten)]
    values: HashMap<String, ParametricValues>,
}

/// A ParametricValues object is a JSON object that represents a collection of parametric values of dynamic non-spatial 
/// attributes that are ascertained at the same times. A parametric value may be a time-varying measure, a sequence of 
/// texts, or a sequence of images. Even though the parametric value may depend on the spatiotemporal location,
/// MF-JSON Prism only considers the temporal dependencies of their changes of value.
///
/// See [7.2.2.1 MF-JSON ParametricValues](https://docs.ogc.org/is/19-045r3/19-045r3.html#pvalues)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
        values: String,
        /// Allowed Interpolations: Discrete, Step
        // TODO enforce?
        interpolation: Option<Interpolation>,
        description: Option<String>,
    },
}
