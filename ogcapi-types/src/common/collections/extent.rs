use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::common::{Bbox, Crs};

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct Extent {
    pub spatial: Option<SpatialExtent>,
    pub temporal: Option<TemporalExtent>,
}

#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct SpatialExtent {
    pub bbox: Option<Vec<Bbox>>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub crs: Option<Crs>,
}

#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct TemporalExtent {
    #[serde_as(as = "Option<Vec<Vec<Option<DisplayFromStr>>>>")]
    pub interval: Option<Vec<Vec<Option<DateTime<Utc>>>>>,
    pub trs: Option<String>,
}
