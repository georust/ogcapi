use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

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

#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SpatialExtent {
    pub bbox: Option<Vec<Bbox>>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub crs: Option<Crs>,
}

impl Default for SpatialExtent {
    fn default() -> Self {
        Self {
            bbox: Some(vec![Bbox::Bbox2D([-180.0, -90.0, 180.0, 90.0])]),
            crs: Some(Crs::from(4326)),
        }
    }
}

#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TemporalExtent {
    #[serde_as(as = "Option<Vec<Vec<Option<DisplayFromStr>>>>")]
    pub interval: Option<Vec<Vec<Option<DateTime<Utc>>>>>,
    pub trs: Option<String>,
}

impl Default for TemporalExtent {
    fn default() -> Self {
        Self {
            interval: Some(vec![vec![None, None]]),
            trs: Default::default(),
        }
    }
}

#[test]
fn extent() {
    let e = serde_json::to_string_pretty(&Extent::default()).unwrap();

    println!("{}", e)
}
