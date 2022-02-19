mod collection;

pub use collection::{Collection, Extent, ItemType, Provider, SpatialExtent};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr};

use crate::common::{
    core::{Datetime, Links},
    crs::Crs,
};

pub static CRS_REF: &str = "#/crs";

#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Collections {
    pub links: Links,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub time_stamp: Option<Datetime>,
    pub number_matched: Option<usize>,
    pub number_returned: Option<usize>,
    pub collections: Vec<Collection>,
    #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
    pub crs: Option<Vec<Crs>>,
}
