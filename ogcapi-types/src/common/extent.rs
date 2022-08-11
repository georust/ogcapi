use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;

use crate::common::{Bbox, Crs};

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Extent {
    pub spatial: Option<SpatialExtent>,
    pub temporal: Option<TemporalExtent>,
}

impl Default for Extent {
    fn default() -> Self {
        Self {
            spatial: Some(SpatialExtent::default()),
            temporal: Some(TemporalExtent::default()),
        }
    }
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SpatialExtent {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bbox: Vec<Bbox>,
    #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    pub crs: Crs,
}

impl Default for SpatialExtent {
    fn default() -> Self {
        Self {
            bbox: vec![Bbox::Bbox2D([-180.0, -90.0, 180.0, 90.0])],
            crs: Default::default(),
        }
    }
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct TemporalExtent {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde_as(as = "Vec<Vec<Option<DisplayFromStr>>>")]
    pub interval: Vec<Vec<Option<DateTime<Utc>>>>,
    #[serde(default = "default_trs")]
    pub trs: String,
}

impl Default for TemporalExtent {
    fn default() -> Self {
        Self {
            interval: vec![vec![None, None]],
            trs: default_trs(),
        }
    }
}

fn default_trs() -> String {
    "http://www.opengis.net/def/uom/ISO-8601/0/Gregorian".to_string()
}
