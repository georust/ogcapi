use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TemporalProperties {
    datetimes: Vec<DateTime<Utc>>,
    values: HashMap<String, TemporalPropertiesValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum TemporalPropertiesValue {
    Measure {
        values: f64,
        interpolation: Option<Interpolation>,
        description: Option<String>,
        form: Option<String>,
    },
    Text {
        values: String,
        interpolation: Option<Interpolation>,
        description: Option<String>,
        form: Option<String>,
    },
    Image {
        values: String,
        interpolation: Option<Interpolation>,
        description: Option<String>,
        form: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Interpolation {
    Discrete,
    Step,
    Linear,
    Regression,
    Url(String),
}
